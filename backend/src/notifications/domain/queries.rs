use crate::notifications::south::port::DomainPort;
use crate::shared::{Failure, Result};
use serde_json::{Value, json};
pub fn is_read(s: &str) -> bool {
    matches!(
        s,
        "messages" | "notification_templates" | "notification_todos"
    )
}
pub fn read<P: DomainPort + ?Sized>(
    s: &P,
    a: &str,
    section: &str,
    _target: &str,
    _now: i64,
) -> Result<Value> {
    match section {
        "messages" => {
            let f = s.facts();
            let mut rows: Vec<Value> = s
                .data()
                .records
                .values()
                .filter(|r| {
                    r["owner"] == a && matches!(r["kind"].as_str(), Some("message" | "consent"))
                })
                .cloned()
                .collect();
            for r in &mut rows {
                let id = r["business_id"].as_str().unwrap_or("");
                let current = match r["target"].as_str() {
                    Some("tasks") => f["tasks"][id]["status"].clone(),
                    Some("withdrawals") => f["applications"][id]["status"].clone(),
                    Some("risk") => f["risk"][id]["status"].clone(),
                    Some("support") => f["cases"]
                        .as_array()
                        .and_then(|v| v.iter().find(|c| c["id"] == id))
                        .map(|c| c["status"].clone())
                        .unwrap_or(Value::Null),
                    _ => Value::Null,
                };
                r["current_status"] = current;
            }
            rows.sort_by_key(|r| std::cmp::Reverse(r["created_at"].as_i64().unwrap_or(0)));
            Ok(json!(rows))
        }
        "notification_templates" => {
            s.authorize(a, "operator", "notifications")?;
            Ok(json!(
                s.data()
                    .records
                    .values()
                    .filter(|r| r["kind"] == "template")
                    .collect::<Vec<_>>()
            ))
        }
        "notification_todos" => {
            let f = s.facts();
            let mut rows = vec![];
            for (collection, role, statuses) in [
                (
                    "tasks",
                    "task_reviewer",
                    vec!["submitted", "awaiting_verification"],
                ),
                (
                    "applications",
                    "manual_handler",
                    vec!["ready", "processing", "unknown", "pending_verification"],
                ),
            ] {
                for r in f[collection]
                    .as_object()
                    .into_iter()
                    .flat_map(|m| m.values())
                {
                    let scope = if collection == "tasks" {
                        r["product_id"].as_str()
                    } else {
                        r["owner"].as_str()
                    }
                    .unwrap_or("");
                    if statuses.contains(&r["status"].as_str().unwrap_or(""))
                        && s.authorize(a, role, scope).is_ok()
                    {
                        rows.push(json!({"id":r["id"],"kind":collection,"status":r["status"]}));
                    }
                }
            }
            for activity in f["campaigns"].as_array().into_iter().flatten() {
                let id = activity["id"].as_str().unwrap_or("");
                if activity["enabled"] == true
                    && activity["used_cents"].as_i64().unwrap_or(0)
                        >= activity["limit_cents"].as_i64().unwrap_or(i64::MAX)
                    && s.authorize(a, "operator", id).is_ok()
                {
                    rows.push(
                        json!({"id":id,"kind":"activity_limit","status":"活动计划数字额度不足"}),
                    );
                }
            }
            Ok(json!(rows))
        }
        _ => Err(Failure("NOT_FOUND", "消息查询不存在")),
    }
}
