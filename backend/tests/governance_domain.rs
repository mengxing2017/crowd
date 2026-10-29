use crate::{
    identity::south::port::rules as identity, platform::south::port::State,
    privacy::south::port::rules as privacy, reporting::south::port::rules as reports,
};
use serde_json::{Value, json};
fn setup() -> State {
    let mut s = State::default();
    for id in ["alice", "bob", "admin", "reviewer", "operator"] {
        s.register_user(id, json!({"time":10}), None);
    }
    for id in ["admin", "reviewer"] {
        let u = s.users.get_mut(id).unwrap();
        u.roles = vec![
            "auditor".into(),
            "identity_reviewer".into(),
            "rule_manager".into(),
            "privacy_manager".into(),
            "manual_handler".into(),
            "operator".into(),
        ];
        u.scope = vec!["*".into()];
    }
    let u = s.users.get_mut("operator").unwrap();
    u.roles = vec!["operator".into()];
    u.scope = vec!["product".into()];
    s
}
fn policy(s: &mut State) {
    privacy::command(s,"admin","privacy_policy_publish",&json!({"version":0,"reason":"隔离测试政策","copy_enabled":true,"closure_enabled":true,"copy_expires_seconds":100,"wechat_receipt_seconds":100,"task_evidence_seconds":100,"settlement_voucher_seconds":100,"support_attachment_seconds":100,"preserve_disputes":true,"retain_financial_history":true,"notice":"隔离测试"}),"policy",100).unwrap();
}
#[test]
fn reports_preserve_watermark_hide_private_data_and_recheck_download_scope() {
    let mut s = setup();
    s.catalog.claims.insert("claim".into(),json!({"id":"claim","product_id":"product","owner":"alice","locked_at":10,"reserved_cents":30}));
    s.catalog.claims.insert("foreign".into(),json!({"id":"foreign","product_id":"secret","owner":"bob","locked_at":10,"reserved_cents":900}));
    s.tasks.instances.insert("claim".into(),json!({"id":"claim","status":"approved","submissions":[{}],"submitted_at":11,"reviewed_at":12}));
    s.commissions.entries.push(json!({"id":"entry","key":"confirm:claim","source":"claim","owner":"alice","kind":"confirmed","amount_cents":30,"at":12}));
    s.withdrawals.applications.insert("app".into(),json!({"id":"app","owner":"alice","status":"ready","created_at":13,"amount_cents":30,"receipt_snapshot":{"account":"private-account","file_id":"private-image"}}));
    let i = json!({"from":0,"until":99,"product_id":"product","reason":"统计核对","expires_seconds":100});
    let r = reports::command(&mut s, "operator", "report_create", &i, "report", 100).unwrap();
    assert_eq!(r["snapshot"]["funnel"]["claims"], 1);
    assert_eq!(r["snapshot"]["products"][0]["confirmed_cents"], 30);
    assert_eq!(r["snapshot"]["funnel"]["settled_applications"], 0);
    assert!(!r.to_string().contains("private-account"));
    let e = reports::command(&mut s, "operator", "export_request", &i, "export", 100).unwrap();
    assert!(e.get("snapshot").is_none());
    assert!(reports::read(&s, "operator", "export_download", "export", 100).is_err());
    reports::process_exports(&mut s, 101);
    assert!(reports::read(&s, "operator", "export_download", "export", 101).is_ok());
    assert!(reports::read(&s, "bob", "export_download", "export", 101).is_err());
    s.users.get_mut("operator").unwrap().scope = vec!["secret".into()];
    assert!(reports::read(&s, "operator", "export_download", "export", 101).is_err());
    assert!(reports::read(&s, "operator", "report_detail", "report", 101).is_err());
    assert_eq!(s.reporting.records["report"], r);
}
#[test]
fn reconciliation_detects_duplicate_keys_and_unknown_results_keep_occupancy() {
    let mut s = setup();
    s.commissions.entries = vec![
        json!({"id":"a","key":"confirm","owner":"alice","kind":"confirmed","amount_cents":30}),
        json!({"id":"b","key":"hold","owner":"alice","kind":"hold","amount_cents":30}),
    ];
    s.withdrawals.applications.insert(
        "app".into(),
        json!({"id":"app","owner":"alice","status":"uncertain","amount_cents":30}),
    );
    let i = json!({"owner":"alice","reason":"核对"});
    let r = reports::command(&mut s, "admin", "reconciliation_run", &i, "matched", 100).unwrap();
    assert_eq!(r["status"], "matched");
    assert_eq!(r["held_cents"], 30);
    s.commissions.entries.push(s.commissions.entries[1].clone());
    let r = reports::command(&mut s, "admin", "reconciliation_run", &i, "difference", 101).unwrap();
    assert_eq!(r["status"], "open");
    assert!(!r["duplicate_entries"].as_array().unwrap().is_empty());
    assert!(reports::command(&mut s, "operator", "reconciliation_run", &i, "bad", 101).is_err());
}
#[test]
fn privacy_copy_is_owned_expires_and_closure_preserves_pending_snapshots() {
    let mut s = setup();
    policy(&mut s);
    s.withdrawals.applications.insert("app".into(),json!({"id":"app","owner":"alice","status":"uncertain","amount_cents":30,"receipt_snapshot":{"file_id":"protected-image"},"registration":{"private_handler_account":"secret"}}));
    let i = json!({"reason":"本人申请"});
    privacy::command(&mut s, "alice", "privacy_copy_request", &i, "copy", 100).unwrap();
    let copy = privacy::read(&s, "alice", "privacy_copy", "copy", 101).unwrap();
    assert!(!copy.to_string().contains("secret"));
    assert!(privacy::read(&s, "bob", "privacy_copy", "copy", 101).is_err());
    assert!(privacy::read(&s, "alice", "privacy_copy", "copy", 200).is_err());
    privacy::command(&mut s, "alice", "privacy_close_request", &i, "close", 100).unwrap();
    let before = s.withdrawals.applications.clone();
    assert!(privacy::command(&mut s,"reviewer","privacy_close_review",&json!({"id":"close","version":1,"approve":true,"retention_confirmed":true,"reason":"复核"}),"x",101).is_err());
    assert_eq!(s.users["alice"].status, "active");
    assert_eq!(s.withdrawals.applications, before);
    assert!(
        privacy::cleanup_candidate(
            &s,
            &json!({"id":"protected-image","purpose":"wechat_receipt","reason":"清理"})
        )
        .is_err()
    );
    assert_eq!(
        privacy::cleanup_candidate(
            &s,
            &json!({"id":"unused-image","purpose":"task_evidence","reason":"清理"})
        )
        .unwrap(),
        100
    );
}
#[test]
fn temporary_grants_require_independent_approval_and_expire_or_revoke() {
    let mut s = setup();
    let future = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1000;
    let i = json!({"user_id":"alice","roles":["auditor","manual_handler"],"scope":["product","alice"],"expires_at":future,"reason":"应急"});
    identity::security_command(&mut s, "admin", "temporary_grant_request", &i, "grant", 100)
        .unwrap();
    let review = json!({"id":"grant","version":1,"approve":true,"reason":"独立审批"});
    assert!(
        identity::security_command(&mut s, "admin", "temporary_grant_review", &review, "x", 101)
            .is_err()
    );
    identity::security_command(
        &mut s,
        "reviewer",
        "temporary_grant_review",
        &review,
        "x",
        101,
    )
    .unwrap();
    assert!(s.authorize("alice", "auditor", "product").is_ok());
    let me = identity::read(&s, "alice", "me", "alice").unwrap();
    assert!(
        me["roles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r == "manual_handler")
    );
    s.withdrawals.applications.insert(
        "temporary-app".into(),
        json!({"id":"temporary-app","owner":"alice","status":"ready","amount_cents":30}),
    );
    let queue =
        crate::withdrawals::south::port::rules::read(&s, "alice", "withdrawal_queue", "alice", 100)
            .unwrap();
    assert_eq!(queue.as_array().unwrap().len(), 1);

    assert!(s.authorize("alice", "auditor", "other").is_err());
    s.security.grants.get_mut("grant").unwrap()["expires_at"] = json!(1);
    assert!(s.authorize("alice", "auditor", "product").is_err());
    s.security.grants.get_mut("grant").unwrap()["expires_at"] = json!(future);
    identity::security_command(
        &mut s,
        "reviewer",
        "temporary_grant_revoke",
        &json!({"id":"grant","version":2,"reason":"撤回"}),
        "x",
        102,
    )
    .unwrap();
    assert!(s.authorize("alice", "auditor", "product").is_err());
    let mut legacy = serde_json::to_value(State::default()).unwrap();
    for key in ["security", "reporting", "privacy"] {
        legacy.as_object_mut().unwrap().remove(key);
    }
    let old = serde_json::from_value::<State>(legacy).unwrap();
    assert!(old.security.grants.is_empty());
    assert!(old.reporting.records.is_empty());
    assert!(old.privacy.records.is_empty());
}
#[test]
fn pagination_keeps_ties_stable_and_legacy_array_shape() {
    let rows = json!([{"id":"b","at":1},{"id":"a","at":1},{"id":"c","at":1}]);
    let page = crate::http::pagination::paginate(rows.clone(), Some(2), None).unwrap();
    assert_eq!(page["items"], json!([{"id":"a","at":1},{"id":"b","at":1}]));
    let next = crate::http::pagination::paginate(rows.clone(), Some(2), Some("b")).unwrap();
    assert_eq!(next["items"], json!([{"id":"c","at":1}]));
    assert_eq!(next["next_cursor"], Value::Null);
    assert_eq!(
        crate::http::pagination::paginate(rows.clone(), None, None).unwrap(),
        rows
    );
    assert!(crate::http::pagination::paginate(rows, Some(0), None).is_err());
}
