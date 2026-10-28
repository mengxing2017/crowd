use super::validation::{not_found, public_category};
use super::*;
use crate::catalog::south::port::DomainPort;
use crate::shared::{Failure, Result, invalid};
use serde_json::{Value, json};

pub trait QueriesRules: DomainPort {
    fn catalog_read(
        &mut self,
        actor: &str,
        section: &str,
        target: &str,
        now: i64,
    ) -> Result<Value> {
        if self.user(actor)?.status != "active" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号当前不可操作"));
        }
        match section {
            "catalog_admin" => {
                let user = self.user(actor)?;
                if !user.roles.iter().any(|r| {
                    ["operator", "product_reviewer", "product_emergency"].contains(&r.as_str())
                }) {
                    return Err(Failure("FORBIDDEN", "无产品管理权限"));
                }
                let all = user.scope.iter().any(|s| s == "*");
                let products: Vec<_> = self
                    .catalog()
                    .products
                    .values()
                    .filter(|p| all || user.scope.iter().any(|s| p["id"] == *s))
                    .cloned()
                    .collect();
                Ok(
                    json!({"categories":self.catalog().categories.values().collect::<Vec<_>>(),"products":products,"revision":self.catalog().revision,"server_time":now}),
                )
            }
            "eligibility" => {
                self.authorize(actor, "identity_reviewer", target)?;
                Ok(self
                    .catalog()
                    .eligibility
                    .get(target)
                    .cloned()
                    .unwrap_or(json!({"version":0})))
            }
            "attribution_policies" => {
                self.authorize(actor, "rule_manager", "platform")?;
                Ok(json!({"policies":self.catalog().policies,"touch_count":0,"disputes":[]}))
            }
            "attribution_admin" => {
                self.authorize(actor, "attribution_reviewer", target)?;
                Ok(
                    json!({"policies":self.catalog().policies,"touch_count":if target=="*" {self.catalog().touches.len()} else {0},"disputes":self.catalog().disputes.values().filter(|d|target=="*" || d["id"]==target).collect::<Vec<_>>()}),
                )
            }
            "my_claims" => Ok(json!(
                self.catalog()
                    .claims
                    .values()
                    .filter(|c| c["owner"] == actor)
                    .collect::<Vec<_>>()
            )),
            "my_attribution" => Ok(
                json!({"touches":self.catalog().touches.iter().filter(|(_,t)|t["user"]==actor).map(|(id,t)| json!({"id":id,"product":t["product"],"time":t["time"],"code":t["code"]})).collect::<Vec<_>>(),"disputes":self.catalog().disputes.values().filter(|d|d["owner"]==actor).collect::<Vec<_>>()}),
            ),
            "catalog_notifications" => Ok(json!(
                self.catalog()
                    .notifications
                    .iter()
                    .filter(|n| n["recipient"] == actor)
                    .rev()
                    .take(100)
                    .collect::<Vec<_>>()
            )),
            _ => Err(not_found()),
        }
    }
    fn visible_release(&self, id: &str, now: i64) -> Result<Value> {
        let p = self.catalog().products.get(id).ok_or(not_found())?;
        if p["archived"] == true || p["offline"] == true {
            return Err(Failure("TARGET_UNAVAILABLE", "产品已下架或归档"));
        }
        let r = p["releases"]
            .as_array()
            .and_then(|r| {
                r.iter().rev().find(|r| {
                    r["status"] == "approved" && r["starts_at"].as_i64().is_some_and(|t| t <= now)
                })
            })
            .ok_or(Failure("TARGET_UNAVAILABLE", "产品尚未发布或已到期"))?;
        if r["ends_at"].as_i64().is_none_or(|t| now >= t) {
            return Err(Failure("TARGET_UNAVAILABLE", "当前发布版本已到期"));
        }
        self.validate_content(&r["content"], now, true)?;
        Ok(r.clone())
    }
    fn public_catalog(&self, query: &Value, now: i64) -> Result<Value> {
        let keyword = query["keyword"].as_str().unwrap_or("").to_lowercase();
        if keyword.len() > 256 {
            return Err(invalid("关键词过长"));
        }
        let minimum = query["min_cents"].as_i64().unwrap_or(0);
        let maximum = query["max_cents"].as_i64().unwrap_or(i64::MAX);
        if query["task_status"]
            .as_str()
            .is_some_and(|s| !["available", "unavailable"].contains(&s))
        {
            return Err(invalid("任务状态筛选无效"));
        }
        if minimum < 0 || maximum < minimum {
            return Err(invalid("佣金筛选范围无效"));
        }
        let mut products = Vec::new();
        for id in self.catalog().products.keys() {
            if let Ok(r) = self.visible_release(id, now) {
                let c = &r["content"];
                let matching: Vec<_> = c["tasks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|t| {
                        let cents = t["commission_cents"].as_i64().unwrap_or(0);
                        let unknown = t["price_preview"]["kind"] == "proportional"
                            && t["price_preview"]["cap_cents"].is_null();
                        !(unknown
                            && (!query["min_cents"].is_null() || !query["max_cents"].is_null()))
                            && minimum <= cents
                            && cents <= maximum
                            && (query["task_status"].as_str().is_none_or(|s| {
                                (s == "available") == self.task_available(id, t, now)
                            }))
                    })
                    .collect();
                if !c["name"]
                    .as_str()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&keyword)
                    || query["category"]
                        .as_str()
                        .is_some_and(|s| c["category"] != s)
                    || matching.is_empty()
                {
                    continue;
                }
                products.push(json!({"id":id,"release_id":r["id"],"content":c,"starts_at":r["starts_at"],"ends_at":r["ends_at"],"available_tasks":matching.iter().filter(|t|self.task_available(id,t,now)).map(|t|t["id"].clone()).collect::<Vec<_>>()}));
            }
        }
        match query["sort"].as_str().unwrap_or("name") {
            "name" => {
                products.sort_by_key(|p| p["content"]["name"].as_str().unwrap_or("").to_owned())
            }
            "commission_asc" | "commission_desc" => {
                products.sort_by_key(|p| {
                    p["content"]["tasks"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(|t| t["commission_cents"].as_i64())
                        .min()
                        .unwrap_or(0)
                });
                if query["sort"] == "commission_desc" {
                    products.reverse();
                }
            }
            _ => return Err(invalid("排序方式无效")),
        }
        let mut categories: Vec<Value> = self
            .catalog()
            .categories
            .values()
            .filter(|c| c["enabled"] == true)
            .map(public_category)
            .collect();
        categories.sort_by(|a, b| {
            (a["sort"].as_i64(), a["id"].as_str()).cmp(&(b["sort"].as_i64(), b["id"].as_str()))
        });
        Ok(json!({"products":products,"categories":categories,"revision":self.catalog().revision}))
    }
}
impl<T: DomainPort + ?Sized> QueriesRules for T {}
