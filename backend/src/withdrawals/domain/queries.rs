use crate::shared::{Failure, Result, text};
use crate::withdrawals::south::port::DomainPort;
use serde_json::{Value, json};
pub(crate) fn mask(mut a: Value) -> Value {
    if let Some(r) = a["receipt_snapshot"].as_object_mut() {
        r.remove("account");
        r.remove("name");
        r.remove("file_id");
    }
    if a["registration"].is_object() {
        a["registration"]["voucher_file_id"] = Value::Null;
    }
    a
}
pub fn read<P: DomainPort + ?Sized>(
    s: &P,
    actor: &str,
    section: &str,
    target: &str,
    now: i64,
) -> Result<Value> {
    s.user(actor)?;
    if section == "withdrawal_rules" {
        if s.authorize(actor, "rule_manager", "platform").is_ok()
            || s.authorize(actor, "verifier", "platform").is_ok()
        {
            return Ok(json!(s.withdrawals().rules));
        }
        return super::active_rule(s);
    }
    let own = section == "my_withdrawals";
    if own && target != actor {
        return Err(Failure("FORBIDDEN", "只能查询本人申请"));
    }
    if section == "withdrawal_detail" {
        let a = s
            .withdrawals()
            .applications
            .get(target)
            .ok_or(Failure("NOT_FOUND", "申请不存在"))?;
        if a["owner"] != actor {
            queue_access(s, actor, text(a, "owner")?)?;
        }
        let mut safe = mask(a.clone());
        safe["source_disputed"] = json!(s.commission_source_disputed(a));
        return Ok(safe);
    }
    if !own {
        let user = s.user(actor)?;
        if !user.roles.iter().any(|r| {
            [
                "settlement_reviewer",
                "manual_handler",
                "verifier",
                "auditor",
            ]
            .contains(&r.as_str())
        }) && !s
            .withdrawals()
            .applications
            .values()
            .any(|a| queue_access(s, actor, a["owner"].as_str().unwrap_or("")).is_ok())
        {
            return Err(Failure("FORBIDDEN", "无队列查询岗位权限"));
        }
    }
    let rows: Vec<_> = s
        .withdrawals()
        .applications
        .values()
        .filter(|a| {
            if own {
                a["owner"] == actor
            } else {
                queue_access(s, actor, a["owner"].as_str().unwrap_or("")).is_ok()
            }
        })
        .map(|a| {
            let source_disputed = s.commission_source_disputed(a);
            let mut a = mask(a.clone());
            a["source_disputed"] = json!(source_disputed);
            let seconds = a["rule"]["reminder_seconds"].as_i64().unwrap_or(0);
            a["overdue_reminder"] = json!(
                seconds > 0
                    && now - a["updated_at"].as_i64().unwrap_or(now) > seconds
                    && !matches!(
                        a["status"].as_str(),
                        Some("settled" | "rejected" | "cancelled")
                    )
            );
            a
        })
        .collect();
    Ok(json!(rows))
}
pub(crate) fn queue_access<P: DomainPort + ?Sized>(s: &P, actor: &str, target: &str) -> Result<()> {
    if s.authorize(actor, "settlement_reviewer", target).is_err()
        && s.authorize(actor, "manual_handler", target).is_err()
        && s.authorize(actor, "verifier", target).is_err()
    {
        s.authorize(actor, "auditor", target)?;
    }
    Ok(())
}
