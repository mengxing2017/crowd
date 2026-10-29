use super::validation::{missing, require};
use crate::shared::{Result, invalid, text};
use crate::tasks::south::port::DomainPort;
use serde_json::{Value, json};

pub trait QueriesRules: DomainPort {
    fn tasks_read(&self, actor: &str, section: &str, target: &str, now: i64) -> Result<Value> {
        require(self.user(actor)?.status == "active", "账号不可查询任务")?;
        match section {
            "task_rules" => {
                require(
                    ["operator", "rule_manager", "verifier", "task_reviewer"]
                        .iter()
                        .any(|r| self.authorize(actor, r, "platform").is_ok()),
                    "无规则管理权限",
                )?;
                Ok(
                    json!({"prices":self.tasks().prices.values().collect::<Vec<_>>(),"templates":self.tasks().templates.values().collect::<Vec<_>>(),"server_time":now}),
                )
            }
            "my_tasks" | "task_queue" => {
                let query: Value = if target.starts_with('{') {
                    serde_json::from_str(target).map_err(|_| invalid("筛选参数错误"))?
                } else {
                    json!({})
                };
                let queue = section == "task_queue";
                if queue {
                    let user = self.user(actor)?;
                    require(
                        user.roles
                            .iter()
                            .any(|r| r == "task_reviewer" || r == "verifier")
                            && !user.scope.is_empty()
                            || self.tasks().instances.values().any(|t| {
                                ["task_reviewer", "verifier"].iter().any(|r| {
                                    self.authorize(actor, r, t["product_id"].as_str().unwrap_or(""))
                                        .is_ok()
                                })
                            }),
                        "无任务审核或复核权限",
                    )?;
                }
                let mut items: Vec<Value> = self
                    .tasks()
                    .instances
                    .values()
                    .filter(|t| {
                        let visible = if queue {
                            let product = t["product_id"].as_str().unwrap_or("");
                            (self.authorize(actor, "task_reviewer", product).is_ok()
                                && matches!(
                                    t["status"].as_str(),
                                    Some("submitted" | "reviewing" | "awaiting_verification")
                                ))
                                || (t["status"] == "awaiting_verification"
                                    && self.authorize(actor, "verifier", product).is_ok())
                        } else {
                            t["owner"] == actor
                        };
                        visible
                            && query["product_id"]
                                .as_str()
                                .is_none_or(|p| t["product_id"] == p)
                            && (query["status"].as_str().is_none_or(|s| t["status"] == s))
                            && query["task_type"]
                                .as_str()
                                .is_none_or(|s| t["template"]["task_type"] == s)
                            && query["risk"]
                                .as_bool()
                                .is_none_or(|r| t["template"]["double_review"] == r)
                    })
                    .map(|t| {
                        let mut item = t.clone();
                        if queue {
                            item["draft"] = Value::Null;
                            item["preview"] = Value::Null;
                            if item["receipt"].is_object() {
                                item["receipt"]["evidence"] = Value::Null;
                            }
                            item["submissions"] = Value::Null;
                            item["waiting_seconds"] =
                                json!(now - t["submitted_at"].as_i64().unwrap_or(now));
                            item["overdue"] = json!(
                                now - t["submitted_at"].as_i64().unwrap_or(now)
                                    > t["template"]["review_sla_seconds"]
                                        .as_i64()
                                        .unwrap_or(i64::MAX)
                            );
                        }
                        item
                    })
                    .collect();
                items.sort_by(|a, b| {
                    (a["submitted_at"].as_i64().unwrap_or(0), a["id"].as_str())
                        .cmp(&(b["submitted_at"].as_i64().unwrap_or(0), b["id"].as_str()))
                });
                if let Some(cursor) = query["cursor"].as_str() {
                    let offset = items
                        .iter()
                        .position(|v| v["id"] == cursor)
                        .ok_or(invalid("游标已失效，请重新查询"))?
                        + 1;
                    items = items.into_iter().skip(offset).collect();
                }
                let limit = query["limit"].as_u64().unwrap_or(20).clamp(1, 50) as usize;
                let more = items.len() > limit;
                items.truncate(limit);
                let cursor = if more {
                    items.last().map(|v| v["id"].clone()).unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                Ok(json!({"items":items,"next_cursor":cursor,"server_time":now}))
            }
            "task_detail" => {
                let t = self.tasks().instances.get(target).ok_or(missing())?;
                if t["owner"] != actor {
                    self.authorize(actor, "task_reviewer", text(t, "product_id")?)
                        .or_else(|_| self.authorize(actor, "verifier", text(t, "product_id")?))?;
                    let mut safe = t.clone();
                    safe["draft"] = Value::Null;
                    safe["preview"] = Value::Null;
                    if safe["receipt"].is_object() {
                        safe["receipt"]["evidence"] = Value::Null;
                    }
                    safe["submissions"] = Value::Null;
                    safe["corrections"]=json!(self.tasks().proposals.values().filter(|p|p["task_id"]==target).map(|p|json!({"id":p["id"],"kind":p["kind"],"status":p["status"],"actor":p["actor"],"reason":p["reason"]})).collect::<Vec<_>>());
                    return Ok(safe);
                }
                Ok(t.clone())
            }
            "task_changes" => {
                self.authorize(actor, "auditor", "platform")?;
                Ok(
                    json!({"confirmations":self.tasks().confirmations,"changes":self.tasks().changes,"inspections":self.tasks().inspections,"proposals":self.tasks().proposals}),
                )
            }
            _ => Err(missing()),
        }
    }
}
impl<T: DomainPort + ?Sized> QueriesRules for T {}
