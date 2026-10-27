//! M05–M07 rules. No HTTP, SQL, payment or commission ledger dependencies.
use crate::community::domain::{Failure, Result, State, check_version, invalid, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Catalog {
    pub categories: BTreeMap<String, Value>,
    pub products: BTreeMap<String, Value>,
    pub policies: Vec<Value>,
    pub touches: BTreeMap<String, Value>,
    pub claims: BTreeMap<String, Value>,
    pub disputes: BTreeMap<String, Value>,
    pub completions: BTreeMap<String, Value>,
    pub notifications: Vec<Value>,
    pub eligibility: BTreeMap<String, Value>,
    pub revision: u64,
}
pub fn is_command(action: &str) -> bool {
    matches!(
        action,
        "category_save"
            | "category_migrate"
            | "product_save"
            | "product_submit"
            | "product_review"
            | "product_offline"
            | "product_archive"
            | "product_rollback"
            | "policy_publish"
            | "touch_bind"
            | "task_claim"
            | "attribution_dispute"
            | "dispute_resolve"
            | "eligibility_verify"
    )
}
pub fn is_read(section: &str) -> bool {
    matches!(
        section,
        "catalog_admin"
            | "attribution_admin"
            | "my_claims"
            | "my_attribution"
            | "catalog_notifications"
            | "attribution_policies"
            | "eligibility"
    )
}
fn num(v: &Value, field: &str) -> Result<i64> {
    v[field]
        .as_i64()
        .filter(|v| v.unsigned_abs() <= 9_007_199_254_740_991)
        .ok_or(invalid("整数或时间字段缺失，金额必须使用安全整数分"))
}
fn strings(v: &Value) -> Result<Vec<String>> {
    serde_json::from_value(v.clone()).map_err(|_| invalid("需提供字符串数组"))
}
fn version(v: &Value) -> u64 {
    v["version"].as_u64().unwrap_or(0)
}
fn not_found() -> Failure {
    Failure("NOT_FOUND", "业务对象不存在")
}
fn conflict() -> Failure {
    Failure("VERSION_CONFLICT", "状态或版本已改变，请刷新")
}
impl State {
    pub fn catalog_access(&self, actor: &str, action: &str, input: &Value) -> Result<()> {
        let user = self.user(actor)?;
        if user.status != "active" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号当前不可操作"));
        }
        let id = input["id"].as_str().unwrap_or("platform");
        match action {
            "category_save" | "category_migrate" => self.authorize(actor, "operator", "platform"),
            "product_save" => self.authorize(actor, "operator", id),
            "product_submit" | "product_archive" | "product_rollback" => {
                self.authorize(actor, "operator", text(input, "id")?)
            }
            "product_offline" if input["emergency"] == true => {
                self.authorize(actor, "product_emergency", text(input, "id")?)
            }
            "product_offline" => self.authorize(actor, "operator", text(input, "id")?),
            "product_review" => self.authorize(actor, "product_reviewer", text(input, "id")?),
            "policy_publish" => self.authorize(actor, "rule_manager", "platform"),
            "dispute_resolve" => self.authorize(actor, "attribution_reviewer", text(input, "id")?),
            "eligibility_verify" => {
                self.authorize(actor, "identity_reviewer", text(input, "user_id")?)
            }
            "task_claim" => self.authorize(actor, "user", actor),
            "touch_bind" | "attribution_dispute" => self.authorize(actor, "user", actor),
            _ => Err(not_found()),
        }
    }
    pub fn catalog_read(
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
                    .catalog
                    .products
                    .values()
                    .filter(|p| all || user.scope.iter().any(|s| p["id"] == *s))
                    .cloned()
                    .collect();
                Ok(
                    json!({"categories":self.catalog.categories.values().collect::<Vec<_>>(),"products":products,"revision":self.catalog.revision,"server_time":now}),
                )
            }
            "eligibility" => {
                self.authorize(actor, "identity_reviewer", target)?;
                Ok(self
                    .catalog
                    .eligibility
                    .get(target)
                    .cloned()
                    .unwrap_or(json!({"version":0})))
            }
            "attribution_policies" => {
                self.authorize(actor, "rule_manager", "platform")?;
                Ok(json!({"policies":self.catalog.policies,"touch_count":0,"disputes":[]}))
            }
            "attribution_admin" => {
                self.authorize(actor, "attribution_reviewer", target)?;
                Ok(
                    json!({"policies":self.catalog.policies,"touch_count":if target=="*" {self.catalog.touches.len()} else {0},"disputes":self.catalog.disputes.values().filter(|d|target=="*" || d["id"]==target).collect::<Vec<_>>()}),
                )
            }
            "my_claims" => Ok(json!(
                self.catalog
                    .claims
                    .values()
                    .filter(|c| c["owner"] == actor)
                    .collect::<Vec<_>>()
            )),
            "my_attribution" => Ok(
                json!({"touches":self.catalog.touches.iter().filter(|(_,t)|t["user"]==actor).map(|(id,t)| json!({"id":id,"product":t["product"],"time":t["time"],"code":t["code"]})).collect::<Vec<_>>(),"disputes":self.catalog.disputes.values().filter(|d|d["owner"]==actor).collect::<Vec<_>>()}),
            ),
            "catalog_notifications" => Ok(json!(
                self.catalog
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
    pub fn reconcile_lifecycle(&mut self, now: i64) {
        let changes: Vec<_> = self
            .catalog
            .products
            .keys()
            .filter_map(|id| {
                let state = match self.visible_release(id, now) {
                    Ok(r) => format!("live:{}", r["id"].as_str().unwrap_or("")),
                    Err(_) => "unavailable".into(),
                };
                (self.catalog.products[id]["observed_state"] != state).then(|| (id.clone(), state))
            })
            .collect();
        for (id, state) in changes {
            self.catalog.products.get_mut(&id).unwrap()["observed_state"] = json!(state);
            self.notification(
                &id,
                if state.starts_with("live:") {
                    "scheduled_live"
                } else {
                    "expired_or_unavailable"
                },
                now,
            );
        }
    }
    pub fn visible_release(&self, id: &str, now: i64) -> Result<Value> {
        let p = self.catalog.products.get(id).ok_or(not_found())?;
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
    pub fn public_catalog(&self, query: &Value, now: i64) -> Result<Value> {
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
        for id in self.catalog.products.keys() {
            if let Ok(r) = self.visible_release(id, now) {
                let c = &r["content"];
                let matching: Vec<_> = c["tasks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|t| {
                        let cents = t["commission_cents"].as_i64().unwrap_or(0);
                        minimum <= cents
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
            .catalog
            .categories
            .values()
            .filter(|c| c["enabled"] == true)
            .map(public_category)
            .collect();
        categories.sort_by(|a, b| {
            (a["sort"].as_i64(), a["id"].as_str()).cmp(&(b["sort"].as_i64(), b["id"].as_str()))
        });
        Ok(json!({"products":products,"categories":categories,"revision":self.catalog.revision}))
    }
    fn validate_content(&self, c: &Value, now: i64, live: bool) -> Result<()> {
        for field in ["name", "category", "summary", "details", "limitations"] {
            text(c, field)?;
        }
        let cat = self
            .catalog
            .categories
            .get(text(c, "category")?)
            .ok_or(invalid("分类不存在"))?;
        if cat["enabled"] != true {
            return Err(invalid("分类已停用"));
        }
        strings(&c["regions"])?;
        strings(&c["qualifications"])?;
        if num(c, "estimated_minutes")? <= 0 {
            return Err(invalid("预计时长必须为正整数"));
        }
        let images = c["images"].as_array().ok_or(invalid("需提供图片数组"))?;
        if images.is_empty() || images.len() > 20 {
            return Err(invalid("需提供 1–20 张产品图片（当前工程上限）"));
        }
        for image in images {
            text(image, "file_id")?;
            text(image, "alt")?;
        }
        let tasks = c["tasks"]
            .as_array()
            .filter(|t| !t.is_empty())
            .ok_or(invalid("至少关联一个任务模板"))?;
        let mut ids = std::collections::BTreeSet::new();
        for t in tasks {
            if !ids.insert(text(t, "id")?) {
                return Err(invalid("任务模板编号重复"));
            }
            for f in ["name", "completion", "evidence", "rule_version"] {
                text(t, f)?;
            }
            if num(t, "version")? <= 0
                || num(t, "commission_cents")? < 0
                || t["currency"] != "CNY"
                || num(t, "submit_seconds")? <= 0
            {
                return Err(invalid("任务版本、整数佣金、币种或提交时限无效"));
            }
            if num(t, "rule_starts_at")? >= num(t, "rule_ends_at")?
                || (live && (now < num(t, "rule_starts_at")? || now >= num(t, "rule_ends_at")?))
            {
                return Err(invalid("佣金规则未生效或已过期"));
            }
            if !t["allow_self"].is_boolean() {
                return Err(invalid("必须明确是否允许自推"));
            }
            for f in ["capacity", "digital_limit_cents"] {
                if !t[f].is_null() && num(t, f)? < 0 {
                    return Err(invalid("名额或计划数字上限无效"));
                }
            }
        }
        Ok(())
    }
    fn task_available(&self, product: &str, task: &Value, now: i64) -> bool {
        let count = self
            .catalog
            .claims
            .values()
            .filter(|c| c["product_id"] == product && c["task"]["id"] == task["id"])
            .count() as i64;
        let used = self
            .catalog
            .claims
            .values()
            .filter(|c| c["product_id"] == product && c["task"]["id"] == task["id"])
            .try_fold(0i64, |acc, c| {
                acc.checked_add(c["task"]["commission_cents"].as_i64().unwrap_or(0))
            });
        task["rule_starts_at"].as_i64().is_some_and(|t| t <= now)
            && task["rule_ends_at"].as_i64().is_some_and(|t| now < t)
            && task["capacity"].as_i64().is_none_or(|n| count < n)
            && task["digital_limit_cents"].as_i64().is_none_or(|n| {
                used.and_then(|u| u.checked_add(task["commission_cents"].as_i64().unwrap_or(0)))
                    .is_some_and(|u| u <= n)
            })
    }
    pub fn collect_touch(&mut self, input: &Value, generated: &str, now: i64) -> Result<Value> {
        // Deliberately no user/promoter input. Ownership is derived from an active server code.
        let object = input.as_object().ok_or(invalid("请求必须为对象"))?;
        if object
            .keys()
            .any(|k| !["code", "scene"].contains(&k.as_str()))
        {
            return Err(invalid("访问仅接受码值与场景"));
        }
        let code = text(input, "code")?;
        text(input, "scene")?;
        let c = self
            .codes
            .get(code)
            .cloned()
            .ok_or(Failure("CODE_UNAVAILABLE", "推广码无效，请返回产品列表"))?;
        if !c.active
            || self.users.get(&c.owner).is_none_or(|u| {
                u.status != "active" || u.restrictions.iter().any(|r| r["scope"] == "promotion")
            })
        {
            return Err(Failure("CODE_UNAVAILABLE", "推广码已失效，请返回产品列表"));
        }
        if let Some(p) = &c.product {
            let r = self.visible_release(p, now)?;
            if c.task.as_ref().is_some_and(|t| {
                !r["content"]["tasks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v["id"] == *t)
            }) {
                return Err(Failure("TARGET_UNAVAILABLE", "推广任务已停用"));
            }
        }
        self.catalog.touches.insert(generated.into(),json!({"id":generated,"code":code,"code_version":c.version,"promoter":c.owner,"product":c.product,"task":c.task,"scene":input["scene"],"time":now,"user":null}));
        Ok(
            json!({"touch_token":generated,"product":c.product,"task":c.task,"notice":"访问不占名额，也不产生佣金数字"}),
        )
    }
    pub fn bind_touch(&mut self, actor: &str, token: &str, now: i64) -> Result<Value> {
        let t = self
            .catalog
            .touches
            .get_mut(token)
            .ok_or(Failure("TOUCH_INVALID", "触点不存在"))?;
        if t["user"].is_string() && t["user"] != actor {
            return Err(Failure("FORBIDDEN", "触点已关联其他身份"));
        }
        // Unbound bearer token is kept only in memory by the client; never returned in query APIs.
        if now < t["time"].as_i64().unwrap_or(now) {
            return Err(invalid("触点时间无效"));
        }
        t["user"] = json!(actor);
        Ok(json!({"status":"bound"}))
    }
    fn select_touch(
        &self,
        actor: &str,
        product: &str,
        task: &str,
        now: i64,
        policy: &Value,
    ) -> Option<Value> {
        let window = policy["window_seconds"].as_i64()?;
        let mut touches: Vec<_> = self
            .catalog
            .touches
            .values()
            .filter(|t| {
                t["user"] == actor
                    && (t["product"].is_null() || t["product"] == product)
                    && (t["task"].is_null() || t["task"] == task)
                    && t["time"]
                        .as_i64()
                        .is_some_and(|at| at <= now && now.saturating_sub(at) <= window)
            })
            .collect();
        touches.sort_by(|a, b| {
            (a["time"].as_i64(), a["id"].as_str()).cmp(&(b["time"].as_i64(), b["id"].as_str()))
        });
        if policy["strategy"] == "first_touch" {
            touches.first()
        } else {
            touches.last()
        }
        .map(|t| (*t).clone())
    }
    fn notification(&mut self, product: &str, kind: &str, now: i64) {
        self.catalog.revision += 1;
        let mut recipients: std::collections::BTreeSet<String> = self
            .users
            .values()
            .filter(|u| {
                u.status == "active"
                    && u.roles.iter().any(|r| {
                        ["operator", "product_reviewer", "product_emergency"].contains(&r.as_str())
                    })
                    && u.scope.iter().any(|s| s == "*" || s == product)
            })
            .map(|u| u.id.clone())
            .collect();
        recipients.extend(
            self.catalog
                .claims
                .values()
                .filter(|c| c["product_id"] == product)
                .filter_map(|c| c["owner"].as_str().map(str::to_owned)),
        );
        for recipient in recipients {
            self.catalog.notifications.push(json!({"recipient":recipient,"product_id":product,"kind":kind,"time":now,"revision":self.catalog.revision}));
        }
    }
    pub fn catalog_command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        now: i64,
    ) -> Result<Value> {
        self.catalog_access(actor, action, input)?;
        let id = input["id"].as_str().unwrap_or(generated).to_owned();
        let result = match action {
            "category_save" => {
                text(input, "name")?;
                num(input, "sort")?;
                text(input, "reason")?;
                if !input["enabled"].is_boolean() {
                    return Err(invalid("需明确分类启停状态"));
                }
                let old = self
                    .catalog
                    .categories
                    .get(&id)
                    .cloned()
                    .unwrap_or(json!({"version":0}));
                check_version(version(&old), input)?;
                let mut history = old["history"].as_array().cloned().unwrap_or_default();
                if version(&old) > 0 {
                    let mut previous = old.clone();
                    previous.as_object_mut().unwrap().remove("history");
                    history.push(previous);
                }
                let mut v = input.clone();
                v["history"] = json!(history);
                v["id"] = json!(id);
                v["version"] = json!(version(&old) + 1);
                self.catalog.categories.insert(id.clone(), v);
                self.catalog.revision += 1;
                json!({"id":id,"version":version(&old)+1})
            }
            "category_migrate" => {
                let to = text(input, "to_category")?;
                text(input, "reason")?;
                if self
                    .catalog
                    .categories
                    .get(to)
                    .is_none_or(|c| c["enabled"] != true)
                {
                    return Err(invalid("目标分类不存在或已停用"));
                }
                let category = self.catalog.categories.get(&id).ok_or(not_found())?;
                check_version(version(category), input)?;
                // Only drafts move. Published versions and historical claims are immutable.
                for p in self.catalog.products.values_mut() {
                    if p["draft"]["category"] == id {
                        p["draft"]["category"] = json!(to);
                        p["version"] = json!(version(p) + 1);
                        p["editor"] = json!(actor);
                    }
                }
                self.catalog.categories.get_mut(&id).unwrap()["version"] =
                    json!(version(category) + 1);
                json!({"status":"drafts_migrated","notice":"已发布版本需重新提交审核"})
            }
            "product_save" => {
                if !input["content"].is_object() {
                    return Err(invalid("产品草稿必须为对象"));
                }
                let p=self.catalog.products.entry(id.clone()).or_insert(json!({"id":id,"version":0,"draft":{},"releases":[],"archived":false,"offline":true}));
                check_version(version(p), input)?;
                if p["archived"] == true {
                    return Err(conflict());
                }
                p["draft"] = input["content"].clone();
                p["version"] = json!(version(p) + 1);
                p["editor"] = json!(actor);
                json!({"id":id,"version":version(p)})
            }
            "product_rollback" => {
                text(input, "reason")?;
                let p = self.catalog.products.get(&id).ok_or(not_found())?;
                check_version(version(p), input)?;
                if p["archived"] == true {
                    return Err(conflict());
                }
                let release = p["releases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| {
                        r["id"] == input["release_id"]
                            && (r["status"] == "approved" || r["status"] == "withdrawn")
                    })
                    .cloned()
                    .ok_or(not_found())?;
                let p = self.catalog.products.get_mut(&id).unwrap();
                p["draft"] = release["content"].clone();
                p["editor"] = json!(actor);
                p["version"] = json!(version(p) + 1);
                json!({"id":id,"version":version(p),"status":"draft","notice":"回滚创建草稿，须重新校验并独立审核发布"})
            }
            "product_submit" => {
                let p = self.catalog.products.get(&id).cloned().ok_or(not_found())?;
                check_version(version(&p), input)?;
                if p["archived"] == true
                    || p["releases"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|r| r["status"] == "pending")
                {
                    return Err(conflict());
                }
                self.validate_content(&p["draft"], now, false)?;
                let start = num(input, "starts_at")?;
                let end = num(input, "ends_at")?;
                if start >= end || end <= now {
                    return Err(invalid("发布有效期无效"));
                }
                for task in p["draft"]["tasks"].as_array().unwrap() {
                    if num(task, "rule_starts_at")? > start || num(task, "rule_ends_at")? < end {
                        return Err(invalid("佣金规则有效期必须覆盖发布有效期"));
                    }
                }
                let r = json!({"id":generated,"content":p["draft"],"starts_at":start,"ends_at":end,"status":"pending","submitted_by":actor,"editor":p["editor"],"submitted_at":now});
                let p = self.catalog.products.get_mut(&id).unwrap();
                p["releases"].as_array_mut().unwrap().push(r);
                p["version"] = json!(version(p) + 1);
                json!({"id":id,"release_id":generated,"version":version(p),"status":"pending"})
            }
            "product_review" => {
                text(input, "reason")?;
                let p = self.catalog.products.get(&id).cloned().ok_or(not_found())?;
                check_version(version(&p), input)?;
                let pos = p["releases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .position(|r| r["id"] == input["release_id"] && r["status"] == "pending")
                    .ok_or(conflict())?;
                let r = &p["releases"][pos];
                if r["editor"] == actor || r["submitted_by"] == actor {
                    return Err(Failure(
                        "FORBIDDEN",
                        "产品编辑、提交与发布审核须由不同人员执行",
                    ));
                }
                if input["approved"] == true {
                    self.validate_content(&r["content"], now, false)?;
                    if num(r, "ends_at")? <= now || p["archived"] == true {
                        return Err(invalid("待审版本已到期或产品已归档"));
                    }
                    for other in p["releases"].as_array().unwrap() {
                        if other["status"] == "approved"
                            && num(other, "starts_at")? < num(r, "ends_at")?
                            && num(r, "starts_at")? < num(other, "ends_at")?
                            && other["starts_at"].as_i64().is_some_and(|t| t > now)
                        {
                            return Err(invalid("与已批准的定时发布有效期冲突"));
                        }
                    }
                } else if input["approved"] != false {
                    return Err(invalid("需明确审核结论"));
                }
                let p = self.catalog.products.get_mut(&id).unwrap();
                p["releases"][pos]["status"] = json!(if input["approved"] == true {
                    "approved"
                } else {
                    "rejected"
                });
                p["releases"][pos]["reviewer"] = json!(actor);
                p["releases"][pos]["review_reason"] = input["reason"].clone();
                if input["approved"] == true {
                    p["offline"] = json!(false);
                }
                p["version"] = json!(version(p) + 1);
                self.notification(&id, "release_reviewed", now);
                json!({"id":id,"version":version(&self.catalog.products[&id])})
            }
            "product_offline" | "product_archive" => {
                text(input, "reason")?;
                if action == "product_offline" && input["stock_policy"] != "continue_existing" {
                    return Err(Failure(
                        "DEPENDENCY_UNAVAILABLE",
                        "当前仅支持停止新领取并保留存量；终止任务需后续任务模块接入",
                    ));
                }
                if input["emergency"] == true {
                    text(input, "risk_id")?;
                    text(input, "impact")?;
                }
                let p = self.catalog.products.get_mut(&id).ok_or(not_found())?;
                check_version(version(p), input)?;
                p["offline"] = json!(true);
                p["version"] = json!(version(p) + 1);
                if action == "product_archive" {
                    p["archived"] = json!(true);
                }
                p["offline_reason"] = input["reason"].clone();
                p["stock_policy"] = json!("continue_existing");
                // Approved schedules are cancelled, never silently resurrected after an emergency.
                for r in p["releases"].as_array_mut().unwrap() {
                    if r["status"] == "approved" {
                        r["status"] = json!("withdrawn");
                    }
                }
                self.notification(&id, action, now);
                json!({"id":id,"version":version(&self.catalog.products[&id]),"stock_policy":"continue_existing"})
            }
            "policy_publish" => {
                let window = num(input, "window_seconds")?;
                if window <= 0 || !["first_touch", "last_touch"].contains(&text(input, "strategy")?)
                {
                    return Err(invalid("明确配置正整数归因窗口及首触或末触策略"));
                }
                check_version(self.catalog.policies.len() as u64, input)?;
                let p = json!({"id":generated,"version":self.catalog.policies.len()+1,"window_seconds":window,"strategy":input["strategy"],"starts_at":now,"owner":actor});
                self.catalog.policies.push(p.clone());
                p
            }
            "touch_bind" => self.bind_touch(actor, text(input, "touch_token")?, now)?,
            "eligibility_verify" => {
                let target = text(input, "user_id")?;
                self.user(target)?;
                if target == actor {
                    return Err(Failure("FORBIDDEN", "不能核验自己的地区或资格"));
                }
                text(input, "reason")?;
                let old = self
                    .catalog
                    .eligibility
                    .get(target)
                    .cloned()
                    .unwrap_or(json!({"version":0,"history":[]}));
                check_version(version(&old), input)?;
                let region = text(input, "region")?;
                strings(&input["qualifications"])?;
                let mut history = old["history"].as_array().cloned().unwrap_or_default();
                history.push(json!({"region":region,"qualifications":input["qualifications"],"verified_by":actor,"reason":input["reason"],"time":now}));
                self.catalog.eligibility.insert(target.into(),json!({"version":version(&old)+1,"region":region,"qualifications":input["qualifications"],"history":history}));
                json!({"version":version(&old)+1})
            }
            "task_claim" => {
                if input["confirmed"] != true {
                    return Err(invalid("须主动确认任务条件和数字佣金"));
                }
                if input
                    .as_object()
                    .unwrap()
                    .keys()
                    .any(|k| !["id", "task_id", "release_id", "confirmed"].contains(&k.as_str()))
                {
                    return Err(invalid("领取不能指定推广人、地区、佣金或身份"));
                }
                let r = self.visible_release(&id, now)?;
                if r["id"] != input["release_id"] {
                    return Err(conflict());
                }
                let task = r["content"]["tasks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|t| t["id"] == input["task_id"])
                    .cloned()
                    .ok_or(not_found())?;
                if self.catalog.claims.values().any(|c| {
                    c["owner"] == actor
                        && c["release_id"] == r["id"]
                        && c["task"]["id"] == task["id"]
                }) {
                    return Err(Failure(
                        "ALREADY_CLAIMED",
                        "本发布版本的任务已领取，请查询原记录",
                    ));
                }
                if !self.task_available(&id, &task, now) {
                    return Err(Failure(
                        "QUOTA_EXHAUSTED",
                        "名额或计划数字上限不足，或规则已到期",
                    ));
                }
                let regions = strings(&r["content"]["regions"])?;
                let required = strings(&r["content"]["qualifications"])?;
                let eligibility = self
                    .catalog
                    .eligibility
                    .get(actor)
                    .cloned()
                    .unwrap_or(Value::Null);
                if (!regions.is_empty() && !regions.iter().any(|s| eligibility["region"] == *s))
                    || required.iter().any(|q| {
                        !eligibility["qualifications"]
                            .as_array()
                            .is_some_and(|v| v.iter().any(|v| v == q))
                    })
                {
                    return Err(Failure("INELIGIBLE", "未满足经核验的地区或资格条件"));
                }
                let policy = self
                    .catalog
                    .policies
                    .last()
                    .cloned()
                    .ok_or(Failure("DEPENDENCY_UNAVAILABLE", "归因策略尚未配置"))?;
                let touch = self.select_touch(actor, &id, text(&task, "id")?, now, &policy);
                if touch.as_ref().is_some_and(|t| t["promoter"] == actor)
                    && task["allow_self"] != true
                {
                    return Err(Failure("SELF_PROMOTION_FORBIDDEN", "此任务不允许本人自推"));
                }
                let category = self
                    .catalog
                    .categories
                    .get(text(&r["content"], "category")?)
                    .cloned()
                    .unwrap();
                let claim = json!({"id":generated,"owner":actor,"product_id":id,"release_id":r["id"],"product":r["content"],"category":public_category(&category),"task":task,"policy":policy,"touch":touch,"locked_at":now,"submit_until":now.checked_add(num(&task,"submit_seconds")?).ok_or(invalid("时限超出范围"))?,"status":"claimed","notice":"领取仅固定条件和归因，不生成可申请佣金"});
                self.catalog.claims.insert(generated.into(), claim.clone());
                claim
            }
            "attribution_dispute" => {
                text(input, "reason")?;
                text(input, "evidence")?;
                let c = self
                    .catalog
                    .claims
                    .get(text(input, "claim_id")?)
                    .ok_or(not_found())?;
                if c["owner"] != actor && c["touch"]["promoter"] != actor {
                    return Err(Failure("FORBIDDEN", "无权申诉此任务"));
                }
                let d = json!({"id":generated,"owner":actor,"claim_id":input["claim_id"],"reason":input["reason"],"evidence":input["evidence"],"original_touch":c["touch"],"status":"pending","version":1,"time":now});
                self.catalog.disputes.insert(generated.into(), d.clone());
                d
            }
            "dispute_resolve" => {
                text(input, "reason")?;
                let decision = text(input, "decision")?;
                if !["uphold", "needs_adjustment"].contains(&decision) {
                    return Err(invalid("裁定只能维持归属或转关联数字调整核查"));
                }
                let d = self.catalog.disputes.get_mut(&id).ok_or(not_found())?;
                check_version(version(d), input)?;
                if d["owner"] == actor {
                    return Err(Failure("FORBIDDEN", "不能裁定自己的申诉"));
                }
                if d["status"] != "pending" {
                    return Err(conflict());
                }
                d["status"] = json!(decision);
                d["reviewer"] = json!(actor);
                d["decision_reason"] = input["reason"].clone();
                d["version"] = json!(version(d) + 1);
                d.clone()
            }
            _ => return Err(not_found()),
        };
        Ok(result)
    }
}
impl crate::catalog::south::port::CompletionRecorder for State {
    fn record_completion(
        &mut self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
    ) -> Result<Value> {
        if source.is_empty() || event.is_empty() || source.len() > 128 || event.len() > 128 {
            return Err(invalid("事件来源或事件编号无效"));
        }
        let key = serde_json::to_string(&(source, event)).unwrap();
        if let Some(previous) = self.catalog.completions.get(&key) {
            if previous["claim_id"] != claim || previous["occurred_at"] != occurred_at {
                return Err(Failure("IDEMPOTENCY_CONFLICT", "完成事件内容冲突"));
            }
            return Ok(previous.clone());
        }
        let c = self.catalog.claims.get(claim).ok_or(not_found())?;
        if occurred_at < num(c, "locked_at")? || occurred_at > num(c, "submit_until")? {
            return Err(invalid("完成事件不在任务有效期限内"));
        }
        if c["touch"]["promoter"] == c["owner"] && c["task"]["allow_self"] != true {
            return Err(Failure("SELF_PROMOTION_FORBIDDEN", "自推归因被禁止"));
        }
        let within = c["touch"].is_null()
            || occurred_at.saturating_sub(c["touch"]["time"].as_i64().unwrap_or(0))
                <= num(&c["policy"], "window_seconds")?;
        let value = json!({"claim_id":claim,"source":source,"event_id":event,"occurred_at":occurred_at,"attribution":if within {c["touch"].clone()} else {Value::Null},"status":if within {"attributed"} else {"outside_window"},"commission_created":false});
        // A second source/event cannot create another effective attribution for the same claim.
        if self
            .catalog
            .completions
            .values()
            .any(|v| v["claim_id"] == claim)
        {
            return Err(Failure("ALREADY_COMPLETED", "此任务已有完成归因记录"));
        }
        self.catalog.completions.insert(key, value.clone());
        Ok(value)
    }
}

fn public_category(category: &Value) -> Value {
    json!({"id":category["id"],"version":category["version"],"name":category["name"],"sort":category["sort"],"enabled":category["enabled"]})
}
