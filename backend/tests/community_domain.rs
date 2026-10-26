use crowd_api::community::domain::State;
use serde_json::{Value, json};
fn state() -> State {
    let mut s = State::default();
    for (id, code) in [
        ("alice", "code-alice"),
        ("bob", "code-bob"),
        ("admin", "code-admin"),
    ] {
        s.register(id, code, json!({"version":"v1"}), None).unwrap();
    }
    let admin = s.users.get_mut("admin").unwrap();
    admin.roles = vec![
        "rule_manager".into(),
        "identity_reviewer".into(),
        "promotion_manager".into(),
        "auditor".into(),
    ];
    admin.scope = vec!["*".into()];
    s
}
fn command(
    s: &mut State,
    who: &str,
    action: &str,
    input: Value,
    id: &str,
) -> Result<Value, crowd_api::community::domain::Failure> {
    s.command(who, action, &input, id, "request-1", "2026-10-27T00:00:00Z")
}
#[test]
fn registration_is_unique_and_collision_never_overwrites_owner() {
    let mut s = state();
    s.register("alice", "new-unused", json!({}), None).unwrap();
    assert!(!s.codes.contains_key("new-unused"));
    assert!(s.register("other", "code-alice", json!({}), None).is_err());
    assert!(!s.users.contains_key("other"));
    assert_eq!(s.codes["code-alice"].owner, "alice");
}
#[test]
fn profile_preserves_identity_and_promotion_and_rejects_stale_version() {
    let mut s = state();
    command(
        &mut s,
        "alice",
        "profile",
        json!({"nickname":"Alice","version":1}),
        "x",
    )
    .unwrap();
    let before = json!(s.users["alice"]);
    assert!(
        command(
            &mut s,
            "alice",
            "profile",
            json!({"nickname":"Overwritten","version":1}),
            "x"
        )
        .is_err()
    );
    assert_eq!(before, json!(s.users["alice"]));
    assert_eq!(s.codes["code-alice"].owner, "alice");
}
#[test]
fn receipts_are_append_only_masked_and_never_change_original_snapshot() {
    let mut s = state();
    command(
        &mut s,
        "alice",
        "receipt",
        json!({"method":"alipay","account":"alice@example.test","confirmed":true,"version":1}),
        "r1",
    )
    .unwrap();
    let original = s.users["alice"].receipts[0].clone();
    command(
        &mut s,
        "alice",
        "receipt",
        json!({"method":"alipay","account":"new@example.test","confirmed":true,"version":2}),
        "r2",
    )
    .unwrap();
    command(
        &mut s,
        "alice",
        "receipt_disable",
        json!({"id":"r1","version":3}),
        "disabled",
    )
    .unwrap();
    assert_eq!(s.users["alice"].receipts[0], original);
    let count = s.users["alice"].receipts.len();
    command(
        &mut s,
        "alice",
        "receipt_disable",
        json!({"id":"r1","version":4}),
        "disabled2",
    )
    .unwrap();
    assert_eq!(count, s.users["alice"].receipts.len());
    let view = s
        .read("alice", "me", "alice", "req", "now")
        .unwrap()
        .to_string();
    assert!(!view.contains("alice@example.test"));
    assert!(!view.contains("new@example.test"));
    assert_eq!(original["status"], "draft");
    assert!(s.read("bob", "receipts", "alice", "req", "now").is_err());
}
#[test]
fn unauthorized_roles_and_data_scope_are_checked() {
    let mut s = state();
    assert!(
        command(
            &mut s,
            "alice",
            "rule_save",
            json!({"kind":"operator","version":0,"content":{}}),
            "r"
        )
        .is_err()
    );
    let admin = s.users.get_mut("admin").unwrap();
    admin.scope = vec!["bob".into()];
    assert!(
        command(
            &mut s,
            "admin",
            "code_rotate",
            json!({"id":"code-alice","version":1,"reason":"leak","approval":"review-1"}),
            "new"
        )
        .is_err()
    );
    assert!(s.read("alice", "audit", "*", "req", "now").is_err());
}
#[test]
fn rotation_preserves_historical_code_and_register_source() {
    let mut s = state();
    s.users.get_mut("bob").unwrap().source = Some("code-alice".into());
    command(
        &mut s,
        "admin",
        "code_rotate",
        json!({"id":"code-alice","version":1,"reason":"leak","approval":"review-1"}),
        "new-code",
    )
    .unwrap();
    assert!(!s.codes["code-alice"].active);
    assert_eq!(s.codes["new-code"].owner, "alice");
    assert_eq!(s.users["bob"].source.as_deref(), Some("code-alice"));
    assert!(s.resolve("code-alice").is_err());
    assert!(s.resolve("new-code").is_ok());
    assert!(
        command(
            &mut s,
            "admin",
            "code_rotate",
            json!({"id":"code-alice","version":1,"reason":"again","approval":"r"}),
            "again"
        )
        .is_err()
    );
}
#[test]
fn rules_keep_versions_and_enforce_digital_boundary() {
    let mut s = state();
    command(
        &mut s,
        "admin",
        "rule_save",
        json!({"kind":"digital_boundary","version":0,"content":{"manual_only":false}}),
        "r1",
    )
    .unwrap();
    assert!(
        command(
            &mut s,
            "admin",
            "rule_publish",
            json!({"id":"r1","version":1}),
            "publish"
        )
        .is_err()
    );
    command(
        &mut s,
        "admin",
        "rule_save",
        json!({"kind":"digital_boundary","version":1,"content":{"manual_only":true}}),
        "r2",
    )
    .unwrap();
    command(
        &mut s,
        "admin",
        "rule_publish",
        json!({"id":"r2","version":2}),
        "pub2",
    )
    .unwrap();
    command(
        &mut s,
        "admin",
        "rule_disable",
        json!({"id":"r2","version":2}),
        "stop",
    )
    .unwrap();
    assert_eq!(s.rules.len(), 2);
    assert_eq!(s.rules[1]["status"], "disabled");
}
#[test]
fn withdrawal_restriction_does_not_block_personal_history_and_click_never_creates_digits() {
    let mut s = state();
    command(
        &mut s,
        "admin",
        "restriction",
        json!({"user_id":"alice","version":1,"reason":"核查","restriction_scope":"withdrawal"}),
        "restriction",
    )
    .unwrap();
    assert!(s.read("alice", "me", "alice", "request", "now").is_ok());
    let before = json!(s.users["alice"]);
    s.resolve("code-alice").unwrap();
    assert_eq!(before, json!(s.users["alice"]));
    command(
        &mut s,
        "admin",
        "restriction",
        json!({"user_id":"alice","version":2,"reason":"推广核查","restriction_scope":"promotion"}),
        "restriction2",
    )
    .unwrap();
    assert!(s.resolve("code-alice").is_err());
}
#[test]
fn unintegrated_dependencies_cannot_delete_accounts_merge_or_create_target_codes() {
    let mut s = state();
    for action in ["closure", "merge", "correction"] {
        command(
            &mut s,
            "alice",
            action,
            json!({"reason":"本人申请"}),
            action,
        )
        .unwrap();
    }
    assert_eq!(s.users["alice"].status, "active");
    assert_eq!(s.cases.len(), 3);
    assert!(s.cases.iter().all(|c| c["status"] != "completed"));
    assert!(
        command(
            &mut s,
            "alice",
            "code_create",
            json!({"product":"missing"}),
            "p"
        )
        .is_err()
    );
    assert!(command(&mut s, "alice", "phone", json!({"phone":"fake"}), "phone").is_err());
}
#[test]
fn rust_user_serialization_matches_platform_neutral_fixture() {
    let mut s = state();
    let actual = s.read("alice", "me", "alice", "req", "now").unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../api-contract/fixtures/community-user.json"
    ))
    .unwrap();
    assert_eq!(actual, expected);
}
#[test]
fn directed_codes_share_owner_and_follow_live_catalogue_status() {
    let mut s = state();
    let catalogue = json!({"products":[{"id":"p1","status":"active","tasks":[{"id":"t1","status":"active","completion":"完成注册","evidence":"注册凭证"},{"id":"t2","status":"active","completion":"完成问卷","evidence":"问卷编号"}]},{"id":"p2","status":"active","tasks":[]}]});
    command(
        &mut s,
        "admin",
        "rule_save",
        json!({"kind":"product_tasks","version":0,"content":catalogue}),
        "catalogue",
    )
    .unwrap();
    command(
        &mut s,
        "admin",
        "rule_publish",
        json!({"id":"catalogue","version":1}),
        "publish",
    )
    .unwrap();
    command(
        &mut s,
        "alice",
        "code_create",
        json!({"product":"p1","task":"t1"}),
        "directed1",
    )
    .unwrap();
    command(
        &mut s,
        "alice",
        "code_create",
        json!({"product":"p2"}),
        "directed2",
    )
    .unwrap();
    assert_eq!(s.codes["directed1"].owner, s.codes["directed2"].owner);
    assert_eq!(s.resolve("directed1").unwrap()["task"], "t1");
    let same = command(
        &mut s,
        "alice",
        "code_create",
        json!({"product":"p1","task":"t1"}),
        "unused",
    )
    .unwrap();
    assert_eq!(same["id"], "directed1");
    assert!(!s.codes.contains_key("unused"));
    assert!(
        command(
            &mut s,
            "alice",
            "code_create",
            json!({"product":"p2","task":"t1"}),
            "bad"
        )
        .is_err()
    );
    let mut updated = catalogue;
    updated["products"][0]["status"] = json!("inactive");
    command(
        &mut s,
        "admin",
        "rule_save",
        json!({"kind":"product_tasks","version":1,"content":updated}),
        "catalogue2",
    )
    .unwrap();
    command(
        &mut s,
        "admin",
        "rule_publish",
        json!({"id":"catalogue2","version":2}),
        "publish2",
    )
    .unwrap();
    assert!(s.resolve("directed1").is_err());
    assert!(s.resolve("directed2").is_ok());
}

#[test]
fn integer_cents_display_never_uses_binary_float() {
    use crowd_api::community::domain::display_cents;
    assert_eq!(display_cents(100), "1.00");
    assert_eq!(display_cents(1), "0.01");
    assert_eq!(display_cents(-1), "-0.01");
    assert_eq!(display_cents(i64::MIN), "-92233720368547758.08");
}
