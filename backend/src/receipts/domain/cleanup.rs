use crate::{
    platform::domain::State,
    shared::{Failure, Result, check_version, text},
};
use serde_json::Value;

pub(crate) fn cleanup_candidate(s: &State, actor: &str, input: &Value) -> Result<(String, i64)> {
    let u = s.user(actor)?;
    check_version(u.version, input)?;
    text(input, "reason")?;
    let id = text(input, "id")?;
    let r = u
        .receipts
        .iter()
        .find(|r| r["id"] == id && r["method"] == "wechat")
        .ok_or(Failure("NOT_FOUND", "本人微信收款资料不存在"))?;
    if !u.receipts.iter().any(|r| r["disables"] == id)
        || s.withdrawals
            .applications
            .values()
            .any(|a| a["receipt_snapshot"]["file_id"] == r["file_id"])
        || s.users
            .values()
            .flat_map(|u| &u.receipts)
            .any(|other| other["id"] != id && other["file_id"] == r["file_id"])
        || s.cases.iter().any(|c| c["owner"] == actor)
    {
        return Err(Failure(
            "RETENTION_PROTECTED",
            "资料仍有效、被历史申请引用或需核查保全，不可清理",
        ));
    }
    let rule = super::policy(s).ok_or(Failure("RULE_NOT_CONFIGURED", "保留政策尚未发布"))?;
    text(rule, "receipt_retention_policy")?;
    let seconds = rule["receipt_retention_seconds"]
        .as_i64()
        .filter(|v| *v > 0)
        .ok_or(Failure(
            "RULE_NOT_CONFIGURED",
            "保留期限尚未明确，不执行清理",
        ))?;
    Ok((text(r, "file_id")?.into(), seconds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn absent_retention_policy_and_open_inquiries_protect_disabled_files() {
        let mut s = State::default();
        s.register_user("alice", json!({}), None);
        s.users.get_mut("alice").unwrap().receipts = vec![
            json!({"id":"r","method":"wechat","file_id":"f"}),
            json!({"id":"stop","disables":"r"}),
        ];
        let input = json!({"id":"r","version":s.users["alice"].version,"reason":"清理"});
        assert_eq!(
            cleanup_candidate(&s, "alice", &input).unwrap_err().0,
            "RULE_NOT_CONFIGURED"
        );
        s.withdrawals.rules.push(json!({"id":"policy","status":"published","receipt_retention_seconds":100,"receipt_retention_policy":"测试保留策略"}));
        assert!(cleanup_candidate(&s, "alice", &input).is_ok());
        s.cases
            .push(json!({"owner":"alice","status":"pending_verification"}));
        assert_eq!(
            cleanup_candidate(&s, "alice", &input).unwrap_err().0,
            "RETENTION_PROTECTED"
        );
        s.cases.clear();
        s.withdrawals.applications.insert(
            "a".into(),
            json!({"status":"unknown","receipt_snapshot":{"file_id":"f"}}),
        );
        assert_eq!(
            cleanup_candidate(&s, "alice", &input).unwrap_err().0,
            "RETENTION_PROTECTED"
        );
    }
}
