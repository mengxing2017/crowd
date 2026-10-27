use crowd_api::{catalog::south::port::CompletionRecorder, community::domain::State};
use serde_json::{Value, json};
fn setup() -> State {
    let mut s = State::default();
    for id in ["editor", "reviewer", "alice", "bob", "promoter", "other"] {
        s.register(id, &format!("code-{id}"), json!({}), None)
            .unwrap();
    }
    s.users.get_mut("editor").unwrap().roles = vec!["operator".into(), "rule_manager".into()];
    s.users.get_mut("editor").unwrap().scope = vec!["*".into()];
    s.users.get_mut("reviewer").unwrap().roles = vec![
        "product_reviewer".into(),
        "product_emergency".into(),
        "attribution_reviewer".into(),
        "identity_reviewer".into(),
    ];
    s.users.get_mut("reviewer").unwrap().scope = vec!["*".into()];
    s.catalog_command(
        "editor",
        "category_save",
        &json!({"id":"cat","version":0,"name":"分类","sort":1,"enabled":true,"reason":"测试"}),
        "category",
        100,
    )
    .unwrap();
    s.catalog_command(
        "editor",
        "policy_publish",
        &json!({"version":0,"window_seconds":1000,"strategy":"last_touch"}),
        "policy",
        100,
    )
    .unwrap();
    s
}
fn content() -> Value {
    json!({"name":"产品","category":"cat","summary":"简介","details":"详情纯文本","limitations":"完成条件须经审核","estimated_minutes":10,"regions":[],"qualifications":[],"images":[{"file_id":"image","alt":"产品说明"}],"tasks":[{"id":"task","name":"任务","version":1,"completion":"完成操作","evidence":"提交证据","rule_version":"rule-1","commission_cents":123,"currency":"CNY","rule_starts_at":100,"rule_ends_at":10000,"allow_self":false,"submit_seconds":2000,"capacity":1,"digital_limit_cents":123}]})
}
fn save(s: &mut State, content: Value, now: i64) {
    let version = s
        .catalog
        .products
        .get("product")
        .map(|p| p["version"].as_u64().unwrap())
        .unwrap_or(0);
    s.catalog_command(
        "editor",
        "product_save",
        &json!({"id":"product","version":version,"content":content}),
        "unused",
        now,
    )
    .unwrap();
}
fn publish(s: &mut State, c: Value, start: i64, end: i64, release: &str) {
    save(s, c, 100);
    let v = s.catalog.products["product"]["version"].clone();
    s.catalog_command(
        "editor",
        "product_submit",
        &json!({"id":"product","version":v,"starts_at":start,"ends_at":end}),
        release,
        100,
    )
    .unwrap();
    let v = s.catalog.products["product"]["version"].clone();
    s.catalog_command("reviewer","product_review",&json!({"id":"product","version":v,"release_id":release,"approved":true,"reason":"内容及任务检查通过"}),"unused",100).unwrap();
}
fn claim(s: &mut State, actor: &str, id: &str, release: &str, now: i64) -> Value {
    s.catalog_command(
        actor,
        "task_claim",
        &json!({"id":"product","task_id":"task","release_id":release,"confirmed":true}),
        id,
        now,
    )
    .unwrap()
}
fn touch(s: &mut State, user: &str, promoter: &str, id: &str, now: i64) {
    s.collect_touch(
        &json!({"code":format!("code-{promoter}"),"scene":"test"}),
        id,
        now,
    )
    .unwrap();
    s.bind_touch(user, id, now).unwrap();
}
#[test]
fn draft_review_schedule_and_expiry_never_expose_unreviewed_content() {
    let mut s = setup();
    publish(&mut s, content(), 200, 900, "release");
    assert!(s.visible_release("product", 199).is_err());
    assert!(s.visible_release("product", 200).is_ok());
    assert!(s.visible_release("product", 900).is_err());
    let mut changed = content();
    changed["name"] = json!("未审草稿");
    save(&mut s, changed, 300);
    assert_eq!(
        s.visible_release("product", 300).unwrap()["content"]["name"],
        "产品"
    );
    assert_eq!(
        s.public_catalog(&json!({"keyword":"未审"}), 300).unwrap()["products"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let version = s.catalog.products["product"]["version"].clone();
    s.catalog_command(
        "editor",
        "product_submit",
        &json!({"id":"product","version":version,"starts_at":300,"ends_at":800}),
        "second",
        300,
    )
    .unwrap();
    save(&mut s, content(), 300);
    assert_eq!(
        s.catalog.products["product"]["releases"][1]["content"]["name"],
        "未审草稿"
    );
    let version = s.catalog.products["product"]["version"].clone();
    s.users
        .get_mut("editor")
        .unwrap()
        .roles
        .push("product_reviewer".into());
    assert_eq!(s.catalog_command("editor","product_review",&json!({"id":"product","version":version,"release_id":"second","approved":true,"reason":"self"}),"unused",300).unwrap_err().0,"FORBIDDEN");
}
#[test]
fn claiming_preserves_snapshots_quota_and_no_automatic_commission() {
    let mut s = setup();
    publish(&mut s, content(), 100, 900, "release");
    touch(&mut s, "alice", "promoter", "touch", 110);
    let c = claim(&mut s, "alice", "claim", "release", 120);
    assert_eq!(c["touch"]["promoter"], "promoter");
    assert_eq!(s.catalog.claims.len(), 1);
    assert!(s.catalog.completions.is_empty());
    touch(&mut s, "alice", "other", "later", 130);
    assert_eq!(s.catalog.claims["claim"]["touch"]["promoter"], "promoter");
    assert!(
        s.catalog_command(
            "bob",
            "task_claim",
            &json!({"id":"product","release_id":"release","task_id":"task","confirmed":true}),
            "claim2",
            140
        )
        .is_err()
    );
    let mut changed = content();
    changed["name"] = json!("新产品");
    save(&mut s, changed, 150);
    let v = s.catalog.products["product"]["version"].clone();
    s.catalog_command(
        "editor",
        "product_offline",
        &json!({"id":"product","version":v,"reason":"下架","stock_policy":"continue_existing"}),
        "unused",
        160,
    )
    .unwrap();
    assert!(s.visible_release("product", 160).is_err());
    assert_eq!(s.catalog.claims["claim"]["product"]["name"], "产品");
    let event = s
        .record_completion("trusted-fixture", "event", "claim", 170)
        .unwrap();
    assert_eq!(event["commission_created"], false);
    assert_eq!(
        s.record_completion("trusted-fixture", "event", "claim", 170)
            .unwrap(),
        event
    );
    assert!(
        s.record_completion("trusted-fixture", "event", "claim", 171)
            .is_err()
    );
    assert!(
        s.record_completion("another", "event", "claim", 170)
            .is_err()
    );
}
#[test]
fn first_last_window_boundaries_self_promotion_and_bearer_binding() {
    let mut s = setup();
    let mut c = content();
    c["tasks"][0]["capacity"] = Value::Null;
    c["tasks"][0]["digital_limit_cents"] = Value::Null;
    publish(&mut s, c, 100, 900, "release");
    touch(&mut s, "alice", "promoter", "one", 110);
    touch(&mut s, "alice", "other", "two", 120);
    assert!(s.bind_touch("bob", "one", 130).is_err());
    assert_eq!(
        claim(&mut s, "alice", "a", "release", 130)["touch"]["promoter"],
        "other"
    );
    s.catalog_command(
        "editor",
        "policy_publish",
        &json!({"version":1,"window_seconds":20,"strategy":"first_touch"}),
        "policy2",
        140,
    )
    .unwrap();
    touch(&mut s, "bob", "promoter", "three", 140);
    touch(&mut s, "bob", "other", "four", 150);
    assert_eq!(
        claim(&mut s, "bob", "b", "release", 160)["touch"]["promoter"],
        "promoter"
    );
    touch(&mut s, "other", "promoter", "expired", 140);
    assert!(claim(&mut s, "other", "c", "release", 161)["touch"].is_null());
    touch(&mut s, "promoter", "promoter", "self", 170);
    assert_eq!(
        s.catalog_command(
            "promoter",
            "task_claim",
            &json!({"id":"product","release_id":"release","task_id":"task","confirmed":true}),
            "self-claim",
            171
        )
        .unwrap_err()
        .0,
        "SELF_PROMOTION_FORBIDDEN"
    );
    assert!(
        s.collect_touch(&json!({"code":"invalid","scene":"x"}), "invalid", 200)
            .is_err()
    );
    assert!(
        s.collect_touch(
            &json!({"code":"code-alice","scene":"x","promoter":"other"}),
            "forged",
            200
        )
        .is_err()
    );
}
#[test]
fn out_of_window_completion_dispute_and_permissions_do_not_transfer_numbers() {
    let mut s = setup();
    publish(&mut s, content(), 100, 900, "release");
    touch(&mut s, "alice", "promoter", "touch", 110);
    claim(&mut s, "alice", "claim", "release", 120);
    assert_eq!(
        s.record_completion("trusted", "event", "claim", 1111)
            .unwrap()["status"],
        "outside_window"
    );
    s.catalog_command(
        "alice",
        "attribution_dispute",
        &json!({"claim_id":"claim","reason":"归因争议","evidence":"访问证据"}),
        "dispute",
        180,
    )
    .unwrap();
    assert!(
        s.catalog_command(
            "bob",
            "attribution_dispute",
            &json!({"claim_id":"claim","reason":"forged","evidence":"x"}),
            "forged",
            180
        )
        .is_err()
    );
    s.catalog_command("reviewer","dispute_resolve",&json!({"id":"dispute","version":1,"decision":"needs_adjustment","reason":"核查后需关联调整"}),"unused",190).unwrap();
    assert_eq!(s.catalog.claims["claim"]["touch"]["promoter"], "promoter");
    assert!(s.catalog_read("alice", "catalog_admin", "*", 190).is_err());
    assert!(
        s.catalog_read("alice", "attribution_admin", "*", 190)
            .is_err()
    );
    s.users.get_mut("editor").unwrap().scope = vec!["another-product".into()];
    assert!(
        s.catalog_access("editor", "product_save", &json!({"id":"product"}))
            .is_err()
    );
}
#[test]
fn restrictions_require_server_verified_eligibility_and_category_history_survives() {
    let mut s = setup();
    let mut c = content();
    c["regions"] = json!(["CN-SH"]);
    c["qualifications"] = json!(["trained"]);
    publish(&mut s, c, 100, 900, "release");
    assert!(
        s.catalog_command(
            "alice",
            "task_claim",
            &json!({"id":"product","release_id":"release","task_id":"task","confirmed":true}),
            "denied",
            120
        )
        .is_err()
    );
    s.catalog_command("reviewer","eligibility_verify",&json!({"user_id":"alice","version":0,"region":"CN-SH","qualifications":["trained"],"reason":"人工核验"}),"unused",120).unwrap();
    claim(&mut s, "alice", "claim", "release", 130);
    s.catalog_command(
        "editor",
        "category_save",
        &json!({"id":"cat","version":1,"name":"分类","sort":1,"enabled":false,"reason":"停用"}),
        "unused",
        140,
    )
    .unwrap();
    assert!(s.visible_release("product", 140).is_err());
    assert_eq!(s.catalog.claims["claim"]["category"]["enabled"], true);
}
#[test]
fn lifecycle_notifications_are_idempotent_and_no_old_release_resurrection() {
    let mut s = setup();
    publish(&mut s, content(), 100, 900, "old");
    publish(&mut s, content(), 200, 400, "new");
    s.reconcile_lifecycle(200);
    let revision = s.catalog.revision;
    s.reconcile_lifecycle(200);
    assert_eq!(revision, s.catalog.revision);
    assert_eq!(s.visible_release("product", 399).unwrap()["id"], "new");
    assert!(s.visible_release("product", 400).is_err());
    s.reconcile_lifecycle(400);
    assert!(s.catalog.revision > revision);
    let revision = s.catalog.revision;
    s.reconcile_lifecycle(401);
    assert_eq!(revision, s.catalog.revision);
}
#[test]
fn missing_rules_images_and_fractional_cents_cannot_be_published() {
    for field in ["name", "tasks", "images"] {
        let mut s = setup();
        let mut c = content();
        c.as_object_mut().unwrap().remove(field);
        save(&mut s, c, 100);
        assert!(
            s.catalog_command(
                "editor",
                "product_submit",
                &json!({"id":"product","version":1,"starts_at":100,"ends_at":900}),
                "release",
                100
            )
            .is_err()
        );
    }
    let mut s = setup();
    let mut c = content();
    c["tasks"][0]["commission_cents"] = json!(1.1);
    save(&mut s, c, 100);
    assert!(
        s.catalog_command(
            "editor",
            "product_submit",
            &json!({"id":"product","version":1,"starts_at":100,"ends_at":900}),
            "release",
            100
        )
        .is_err()
    );
}

#[test]
fn rust_public_catalog_matches_shared_typescript_fixture() {
    let mut s = setup();
    publish(&mut s, content(), 100, 900, "release");
    let fixture: Value =
        serde_json::from_str(include_str!("../../api-contract/fixtures/catalog.json")).unwrap();
    assert_eq!(s.public_catalog(&json!({}), 120).unwrap(), fixture);
}

#[test]
fn public_categories_are_sorted_and_exclude_internal_notes_and_closed_codes_fall_back() {
    let mut s = setup();
    publish(&mut s, content(), 100, 900, "release");
    s.catalog_command("editor","category_save",&json!({"id":"first","version":0,"name":"排前分类","sort":0,"enabled":true,"reason":"内部核查原因"}),"unused",120).unwrap();
    let public = s.public_catalog(&json!({}), 120).unwrap();
    assert_eq!(public["categories"][0]["id"], "first");
    assert!(!public.to_string().contains("内部核查原因"));
    assert!(public["categories"][1]["history"].is_null());
    s.users.get_mut("promoter").unwrap().status = "closed".into();
    assert_eq!(
        s.collect_touch(
            &json!({"code":"code-promoter","scene":"test"}),
            "closed-visit",
            130
        )
        .unwrap_err()
        .0,
        "CODE_UNAVAILABLE"
    );
}
