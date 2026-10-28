#[cfg(test)]
mod tests {
    use crate::platform::south::port::State;
    use crate::receipts::domain::cleanup_candidate;
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
