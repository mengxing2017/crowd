use crate::community::domain::{Failure, Result, State, invalid, text};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
mod execution;
mod review;
mod rules;
pub use rules::calculate;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Tasks {
    pub reminders: Vec<String>,
    pub attachments: BTreeMap<String, Value>,
    pub verified_results: BTreeMap<String, Value>,
    pub prices: BTreeMap<String, Value>,
    pub templates: BTreeMap<String, Value>,
    pub instances: BTreeMap<String, Value>,
    pub orders: BTreeMap<String, String>,
    pub confirmations: BTreeMap<String, Value>,
    pub changes: Vec<Value>,
    pub inspections: Vec<Value>,
    pub proposals: BTreeMap<String, Value>,
}
pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "price_save"
            | "price_review"
            | "template_save"
            | "template_review"
            | "template_disable"
            | "task_start"
            | "task_step"
            | "task_abandon"
            | "task_pause"
            | "task_resume"
            | "task_expire"
            | "evidence_discard"
            | "evidence_save"
            | "evidence_preview"
            | "evidence_submit"
            | "evidence_withdraw"
            | "review_claim"
            | "review_decide"
            | "review_verify"
            | "review_batch_preview"
            | "review_batch"
            | "task_repair_propose"
            | "task_repair_review"
            | "review_revoke_propose"
            | "review_revoke_review"
            | "quality_inspect"
            | "evidence_reveal"
    )
}
pub fn is_read(s: &str) -> bool {
    matches!(
        s,
        "task_rules" | "my_tasks" | "task_queue" | "task_detail" | "task_changes"
    )
}
fn number(v: &Value, field: &str) -> Result<i64> {
    v[field]
        .as_i64()
        .filter(|v| v.unsigned_abs() <= 9_007_199_254_740_991)
        .ok_or(invalid("金额、数量和时间须为安全整数"))
}
fn positive(v: &Value, field: &str) -> Result<i64> {
    number(v, field).and_then(|n| {
        if n > 0 {
            Ok(n)
        } else {
            Err(invalid("时限和数量必须大于零"))
        }
    })
}
fn conflict() -> Failure {
    Failure("VERSION_CONFLICT", "对象或状态已改变，请刷新后重试")
}
fn missing() -> Failure {
    Failure("NOT_FOUND", "任务业务对象不存在")
}
fn require(condition: bool, message: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}
fn version(v: &Value) -> u64 {
    v["version"].as_u64().unwrap_or(0)
}
fn array<'a>(v: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    v[field].as_array().ok_or(invalid("数组字段缺失"))
}
fn next_time(now: i64, seconds: i64) -> Result<i64> {
    now.checked_add(seconds)
        .filter(|n| *n <= 9_007_199_254_740_991)
        .ok_or(invalid("时间超出范围"))
}
fn active(status: &Value) -> bool {
    matches!(
        status.as_str(),
        Some(
            "claimed"
                | "in_progress"
                | "supplement"
                | "submitted"
                | "reviewing"
                | "awaiting_verification"
                | "paused"
        )
    )
}

impl State {
    pub fn tasks_access(&self, actor: &str, action: &str, input: &Value) -> Result<()> {
        require(self.user(actor)?.status == "active", "账号当前不可处理任务")?;
        let id = input["id"].as_str().unwrap_or("platform");
        let target = self
            .tasks
            .instances
            .get(id)
            .and_then(|v| v["product_id"].as_str())
            .unwrap_or("platform");
        match action {
            "price_save" => self.authorize(actor, "rule_manager", "platform"),
            "price_review" => self.authorize(actor, "verifier", "platform"),
            "template_save" | "template_disable" => self.authorize(actor, "operator", "platform"),
            "template_review" => self.authorize(actor, "verifier", "platform"),
            "task_pause" | "task_resume" | "task_expire" | "task_repair_propose" => {
                self.authorize(actor, "operator", target)
            }
            "review_verify" | "task_repair_review" | "review_revoke_review" => {
                self.authorize(actor, "verifier", target)
            }
            "review_claim" | "review_decide" | "review_revoke_propose" | "quality_inspect" => {
                self.authorize(actor, "task_reviewer", target)
            }
            "evidence_reveal" => {
                text(input, "reason")?;
                let t = self.tasks.instances.get(id).ok_or(missing())?;
                let assigned = self.authorize(actor, "task_reviewer", target).is_ok()
                    && t["reviewer"] == actor
                    && matches!(
                        t["status"].as_str(),
                        Some("reviewing" | "awaiting_verification")
                    );
                if !assigned && self.authorize(actor, "verifier", target).is_err() {
                    return Err(Failure("FORBIDDEN", "无本单敏感证据查看权限"));
                }
                Ok(())
            }
            "review_batch_preview" | "review_batch" => {
                for entry in array(input, "entries")? {
                    let task = self
                        .tasks
                        .instances
                        .get(text(entry, "id")?)
                        .ok_or(missing())?;
                    self.authorize(actor, "task_reviewer", text(task, "product_id")?)?;
                }
                Ok(())
            }
            _ => {
                self.authorize(actor, "user", actor)?;
                require(
                    self.tasks
                        .instances
                        .get(id)
                        .is_some_and(|v| v["owner"] == actor),
                    "只能处理本人任务",
                )
            }
        }
    }
    pub fn tasks_read(&self, actor: &str, section: &str, target: &str, now: i64) -> Result<Value> {
        require(self.user(actor)?.status == "active", "账号不可查询任务")?;
        match section {
            "task_rules" => {
                let u = self.user(actor)?;
                require(
                    u.roles.iter().any(|r| {
                        ["operator", "rule_manager", "verifier", "task_reviewer"]
                            .contains(&r.as_str())
                    }) && u.scope.iter().any(|s| s == "*" || s == "platform"),
                    "无规则管理权限",
                )?;
                Ok(
                    json!({"prices":self.tasks.prices.values().collect::<Vec<_>>(),"templates":self.tasks.templates.values().collect::<Vec<_>>(),"server_time":now}),
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
                            && !user.scope.is_empty(),
                        "无任务审核或复核权限",
                    )?;
                }
                let mut items: Vec<Value> = self
                    .tasks
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
                let t = self.tasks.instances.get(target).ok_or(missing())?;
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
                    safe["corrections"]=json!(self.tasks.proposals.values().filter(|p|p["task_id"]==target).map(|p|json!({"id":p["id"],"kind":p["kind"],"status":p["status"],"actor":p["actor"],"reason":p["reason"]})).collect::<Vec<_>>());
                    return Ok(safe);
                }
                Ok(t.clone())
            }
            "task_changes" => {
                self.authorize(actor, "auditor", "platform")?;
                Ok(
                    json!({"confirmations":self.tasks.confirmations,"changes":self.tasks.changes,"inspections":self.tasks.inspections,"proposals":self.tasks.proposals}),
                )
            }
            _ => Err(missing()),
        }
    }
    pub fn tasks_command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        now: i64,
    ) -> Result<Value> {
        self.tasks_access(actor, action, input)?;
        if action.starts_with("price_") || action.starts_with("template_") {
            return self.rule_command(actor, action, input, generated, now);
        }
        let mut result = if action.starts_with("review_")
            || action == "quality_inspect"
            || action.starts_with("task_repair_")
            || action == "evidence_reveal"
        {
            self.review_command(actor, action, input, generated, now)?
        } else {
            self.execution_command(actor, action, input, generated, now)?
        };
        if action != "evidence_reveal"
            && result["owner"].as_str().is_some_and(|owner| owner != actor)
        {
            result["draft"] = Value::Null;
            result["preview"] = Value::Null;
            result["submissions"] = Value::Null;
            if result["receipt"].is_object() {
                result["receipt"]["evidence"] = Value::Null;
            }
        }
        Ok(result)
    }
}
