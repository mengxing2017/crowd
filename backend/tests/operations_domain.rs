use crate::{
    campaigns::south::port::rules as campaigns,
    commissions::south::port::rules::{self as ledger, adjustments},
    notifications::south::port::rules as messages,
    platform::south::port::State,
    risk::south::port::rules as risk,
    support::south::port::rules::tickets,
};
use serde_json::{Value, json};
fn setup() -> State {
    let mut s = State::default();
    for id in [
        "alice", "bob", "operator", "reviewer", "verifier", "support", "risk",
    ] {
        s.register_user(id, json!({}), None);
    }
    for (id, roles) in [
        ("operator", vec!["operator", "rule_manager"]),
        ("reviewer", vec!["settlement_reviewer"]),
        ("verifier", vec!["verifier"]),
        ("support", vec!["support"]),
        ("risk", vec!["risk_manager"]),
    ] {
        let u = s.users.get_mut(id).unwrap();
        u.roles = roles.into_iter().map(String::from).collect();
        u.scope = vec!["*".into()];
    }
    s.catalog
        .products
        .insert("product".into(), json!({"id":"product"}));
    s
}
fn task(s: &mut State, id: &str, approved: bool) {
    let t = json!({"id":id,"owner":"alice","product_id":"product","version":1,"status":if approved{"approved"}else{"submitted"},"reserved_cents":1000,"submissions":[{"id":"evidence"}],"events":[],"task":{"commission_cents":1000}});
    s.tasks.instances.insert(id.into(), t.clone());
    s.catalog.claims.insert(id.into(), t);
    if approved {
        s.tasks.confirmations.insert(id.into(),json!({"id":id,"claim_id":id,"owner":"alice","calculation":{"amount_cents":1000,"base_cents":2000},"price":{"rounding":"half_up"}}));
        ledger::sync_tasks(s, "reviewer", 100).unwrap();
    }
}
fn proposal(s: &mut State, id: &str, restores: Value) -> Value {
    adjustments::command(s,"reviewer","adjustment_propose",&json!({"owner":"alice","task_id":"task","basis_key":id,"source":"trusted_order","evidence":"可信失效说明","amount_cents":400,"restores":restores,"reason":"成果部分失效"}),id,101).unwrap()
}
#[test]
fn adjustments_append_decrease_and_restore_without_rewriting_original() {
    let mut s = setup();
    task(&mut s, "task", true);
    let original = s.commissions.entries.clone();
    let p = proposal(&mut s, "revoke", Value::Null);
    assert!(
        adjustments::command(
            &mut s,
            "reviewer",
            "adjustment_review",
            &json!({"id":p["id"],"version":1,"approve":true,"reason":"自审"}),
            "x",
            102
        )
        .is_err()
    );
    adjustments::command(
        &mut s,
        "verifier",
        "adjustment_review",
        &json!({"id":p["id"],"version":1,"approve":true,"reason":"独立核查"}),
        "x",
        102,
    )
    .unwrap();
    assert_eq!(&s.commissions.entries[..original.len()], &original);
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        600
    );
    let r = proposal(&mut s, "restore", json!("revoke"));
    adjustments::command(
        &mut s,
        "verifier",
        "adjustment_review",
        &json!({"id":r["id"],"version":1,"approve":true,"reason":"撤回失效依据"}),
        "x",
        103,
    )
    .unwrap();
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1000
    );
    assert!(
        adjustments::command(
            &mut s,
            "verifier",
            "adjustment_review",
            &json!({"id":r["id"],"version":1,"approve":true,"reason":"重复"}),
            "x",
            104
        )
        .is_err()
    );
    assert!(adjustments::command(&mut s,"reviewer","adjustment_propose",&json!({"owner":"alice","task_id":"task","basis_key":"restore2","source":"trusted","evidence":"重复恢复","amount_cents":1,"restores":"revoke","reason":"重复"}),"restore2",105).is_err());
}
#[test]
fn pending_cancel_does_not_reduce_other_confirmed_commissions() {
    let mut s = setup();
    task(&mut s, "other", true);
    task(&mut s, "task", false);
    let p = proposal(&mut s, "cancel", Value::Null);
    adjustments::command(
        &mut s,
        "verifier",
        "adjustment_review",
        &json!({"id":p["id"],"version":1,"approve":true,"reason":"待审失效"}),
        "x",
        102,
    )
    .unwrap();
    assert_eq!(s.tasks.instances["task"]["status"], "invalidated");
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1000
    );
    assert_eq!(ledger::summary(&s, "alice").unwrap()["pending_cents"], 0);
}
#[test]
fn held_and_settled_adjustments_enter_dispute_without_releasing_or_clawback() {
    for status in ["processing", "settled"] {
        let mut s = setup();
        task(&mut s, "task", true);
        s.withdrawals.applications.insert("app".into(),json!({"id":"app","owner":"alice","status":status,"amount_cents":800,"version":1,"events":[],"handler":"handler","receipt_snapshot":{"id":"original","version":1}}));
        ledger::append(
            &mut s,
            "withdrawal:app:hold",
            "alice",
            "hold",
            800,
            "app",
            "alice",
            100,
            json!({"sources":[{"source":"task","amount_cents":800}]}),
        )
        .unwrap();
        if status == "settled" {
            ledger::append(
                &mut s,
                "withdrawal:app:settled",
                "alice",
                "settled",
                800,
                "app",
                "handler",
                100,
                json!({}),
            )
            .unwrap();
        }
        let before = s.withdrawals.applications["app"]["receipt_snapshot"].clone();
        let p = proposal(&mut s, "hold", Value::Null);
        let r = adjustments::command(
            &mut s,
            "verifier",
            "adjustment_review",
            &json!({"id":p["id"],"version":1,"approve":true,"reason":"先核查"}),
            "x",
            102,
        )
        .unwrap();
        assert_eq!(r["status"], "awaiting_resolution");
        assert_eq!(
            s.withdrawals.applications["app"]["receipt_snapshot"],
            before
        );
        assert!(
            !s.commissions
                .entries
                .iter()
                .any(|e| e["kind"] == "decrease")
        );
        assert_eq!(ledger::summary(&s, "alice").unwrap()["consistent"], true);
        if status == "processing" {
            assert_eq!(
                s.withdrawals.applications["app"]["adjustment_review_required"],
                true
            );
        }
        crate::withdrawals::south::port::resume_after_dispute(&mut s, "verifier", 103).unwrap();
        if status == "processing" {
            assert_eq!(
                s.withdrawals.applications["app"]["adjustment_review_required"],
                true
            );
        }
        ledger::command(&mut s,"verifier","commission_dispute_resolve",&json!({"id":"adjustment:hold","confirmed":true,"reason":"明确人工核查结论，不自动追偿"}),"resolve",104).unwrap();
        s.users.get_mut("verifier").unwrap().scope = vec!["bob".into()];
        crate::withdrawals::south::port::resume_after_dispute(&mut s, "verifier", 105).unwrap();
        if status == "processing" {
            assert_eq!(
                s.withdrawals.applications["app"]["adjustment_review_required"],
                true
            );
        }
        s.users.get_mut("verifier").unwrap().scope = vec!["*".into()];
        crate::withdrawals::south::port::resume_after_dispute(&mut s, "verifier", 106).unwrap();
        if status == "processing" {
            assert_eq!(
                s.withdrawals.applications["app"]["adjustment_review_required"],
                false
            );
            assert_eq!(s.withdrawals.applications["app"]["handler"], "handler");
            assert_eq!(
                s.withdrawals.applications["app"]["receipt_snapshot"],
                before
            );
        } else {
            let closed=adjustments::command(&mut s,"verifier","adjustment_review",&json!({"id":p["id"],"version":r["version"],"approve":true,"reason":"争议结论记录结案"}),"close",107).unwrap();
            assert_eq!(closed["status"], "dispute_recorded");
            assert!(
                !s.commissions
                    .entries
                    .iter()
                    .any(|e| e["kind"] == "decrease")
            );
        }
    }
}
#[test]
fn partial_invalidity_is_integer_deterministic_and_original_basis_is_unique() {
    let mut s = setup();
    task(&mut s, "task", true);
    let i = json!({"owner":"alice","task_id":"task","basis_key":"partial","source":"order","evidence":"可信原成果","amount_cents":1,"invalid_base_cents":801,"reason":"部分失效"});
    let p =
        adjustments::command(&mut s, "reviewer", "adjustment_propose", &i, "partial", 100).unwrap();
    assert_eq!(p["amount_cents"], 401);
    assert!(
        adjustments::command(&mut s, "reviewer", "adjustment_propose", &i, "other", 100).is_err()
    );
}
#[test]
fn campaign_optional_limits_statistics_and_archive_preserve_history() {
    let mut s = setup();
    let config = json!({"id":"activity","version":0,"name":"测试活动","responsible":"operator","products":["product"],"enabled":false,"reason":"测试配置"});
    campaigns::command(&mut s, "operator", "campaign_save", &config, "x", 100).unwrap();
    task(&mut s, "task", false);
    campaigns::check_claim(&s, &json!({"product_id":"product"})).unwrap();
    let mut limit = config.clone();
    limit["version"] = json!(1);
    limit["enabled"] = json!(true);
    limit["limit_cents"] = json!(1000);
    campaigns::command(&mut s, "operator", "campaign_save", &limit, "x", 101).unwrap();
    task(&mut s, "second", false);
    assert!(campaigns::check_claim(&s, &json!({"product_id":"product"})).is_err());
    let r = campaigns::command(
        &mut s,
        "operator",
        "campaign_archive",
        &json!({"id":"activity","version":2,"reason":"存量继续由运营负责","confirmed":true}),
        "x",
        102,
    )
    .unwrap();
    assert_eq!(r["status"], "archived");
    assert_eq!(s.tasks.instances.len(), 2);
    assert!(campaigns::check_claim(&s, &json!({"product_id":"product"})).is_err());
}
#[test]
fn notifications_are_deduplicated_owned_and_display_live_status() {
    let mut s = setup();
    task(&mut s, "task", false);
    messages::sync(&mut s, 100);
    messages::sync(&mut s, 101);
    assert_eq!(s.notifications.records.len(), 1);
    let m = messages::read(&s, "alice", "messages", "alice", 101).unwrap()[0].clone();
    assert!(
        messages::command(
            &mut s,
            "bob",
            "notification_read",
            &json!({"id":m["id"],"version":1}),
            "x",
            102
        )
        .is_err()
    );
    s.tasks.instances.get_mut("task").unwrap()["status"] = json!("approved");
    assert_eq!(
        messages::read(&s, "alice", "messages", "alice", 102).unwrap()[0]["current_status"],
        "approved"
    );
    messages::command(
        &mut s,
        "alice",
        "notification_consent",
        &json!({"template":"updates","version":0,"granted":false,"scene":"center"}),
        "x",
        103,
    )
    .unwrap();
    assert!(
        !messages::read(&s, "alice", "messages", "alice", 103)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}
#[test]
fn tickets_reject_foreign_links_hide_internal_notes_and_require_independent_appeals() {
    let mut s = setup();
    task(&mut s, "task", false);
    s.tasks.instances.get_mut("task").unwrap()["status"] = json!("rejected");
    s.tasks.instances.get_mut("task").unwrap()["reviewer"] = json!("support");
    s.tasks.instances.get_mut("task").unwrap()["decisions"] = json!([{"actor":"support","at":100}]);
    tickets::command(&mut s,"operator","support_policy",&json!({"version":0,"reason":"隔离测试政策","appeal_seconds":100,"max_appeals":2,"reopen_seconds":100,"independent":true}),"policy",100).unwrap();
    let input = json!({"category":"appeal","related_id":"task","description":"请独立复核","evidence":"新证据"});
    assert!(tickets::command(&mut s, "bob", "ticket_create", &input, "bad", 101).is_err());
    tickets::command(&mut s, "alice", "ticket_create", &input, "ticket", 101).unwrap();
    tickets::command(&mut s,"support","ticket_assign",&json!({"id":"ticket","version":1,"assigned_to":"support","queue":"review","due_at":200,"reason":"分配"}),"x",102).unwrap();
    tickets::command(
        &mut s,
        "support",
        "ticket_reply",
        &json!({"id":"ticket","version":2,"body":"内部核查说明","internal":true,"reason":"核查"}),
        "reply",
        103,
    )
    .unwrap();
    assert_eq!(
        tickets::read(&s, "alice", "tickets", "alice").unwrap()[0]["replies"],
        json!([])
    );
    assert_eq!(
        tickets::read(&s, "support", "tickets", "support").unwrap()[0]["replies"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        tickets::command(
            &mut s,
            "support",
            "ticket_close",
            &json!({"id":"ticket","version":3,"reason":"复核","conclusion":"维持原结论"}),
            "x",
            104
        )
        .is_err()
    );
    assert!(s.commissions.entries.is_empty());
}
#[test]
fn risk_rules_expire_and_observation_does_not_change_commissions() {
    let mut s = setup();
    task(&mut s, "task", true);
    let rule = json!({"id":"frequency","version":0,"signal":"claim_frequency","enabled":true,"starts_at":50,"ends_at":200,"window_seconds":100,"threshold":1,"purpose":"测试领取频率","evaluation":"仅隔离测试参数","reason":"显式配置"});
    risk::command(&mut s, "risk", "risk_rule_publish", &rule, "x", 100).unwrap();
    s.catalog.claims.get_mut("task").unwrap()["locked_at"] = json!(100);
    assert!(risk::check(&s, "alice", "task_claim", 101).is_err());
    assert!(risk::check(&s, "alice", "task_claim", 201).is_ok());
    s.catalog.claims.insert(
        "second".into(),
        json!({"id":"second","owner":"alice","locked_at":100}),
    );
    let original = s.commissions.entries.clone();
    risk::scan(&mut s, 101);
    assert_eq!(s.commissions.entries, original);
    assert!(s.risk.records.values().any(|r| r["kind"] == "case"));
}

#[test]
fn workflow_serialization_matches_typescript_fixture() {
    let mut s = setup();
    campaigns::command(&mut s,"operator","campaign_save",&json!({"id":"activity","version":0,"name":"测试活动","responsible":"operator","products":["product"],"enabled":false,"reason":"fixture"}),"x",100).unwrap();
    task(&mut s, "task", false);
    messages::sync(&mut s, 100);
    tickets::command(
        &mut s,
        "alice",
        "ticket_create",
        &json!({"category":"registration","description":"注册问题"}),
        "ticket",
        100,
    )
    .unwrap();
    let project = |value: Value, fields: &[&str]| {
        json!(
            value
                .as_array()
                .unwrap()
                .iter()
                .map(|v| fields
                    .iter()
                    .map(|k| (k.to_string(), v[k].clone()))
                    .collect::<serde_json::Map<_, _>>())
                .collect::<Vec<_>>()
        )
    };
    let actual = json!({"campaigns":project(campaigns::read(&s,"operator","campaigns","operator",100).unwrap(),&["id","version","enabled","status"]),"messages":project(messages::read(&s,"alice","messages","alice",100).unwrap(),&["id","version","business_id","status","current_status"]),"tickets":project(tickets::read(&s,"alice","tickets","alice").unwrap(),&["id","version","owner","category","status"])});
    let fixture: Value =
        serde_json::from_str(include_str!("../../api-contract/fixtures/operations.json")).unwrap();
    assert_eq!(actual, fixture);
}

#[test]
fn notification_delivery_requires_consent_and_stops_after_configured_attempts() {
    use crate::notifications::south::port::DeliveryOutcome;
    let mut s = setup();
    messages::command(&mut s,"operator","notification_template",&json!({"id":"task-template","version":0,"title":"任务状态","body":"{business_id}:{status}:{notice}","variables":["business_id","status","notice"],"target":"tasks","max_attempts":2,"retry_seconds":10,"preview_confirmed":true,"reason":"隔离测试"}),"template",90).unwrap();
    task(&mut s, "task", false);
    messages::sync(&mut s, 100);
    let id = s
        .notifications
        .records
        .values()
        .find(|r| r["kind"] == "message")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(messages::delivery::reserve(&mut s, &id, "attempt1", 100).is_err());
    messages::command(
        &mut s,
        "alice",
        "notification_consent",
        &json!({"template":"task-template","version":0,"granted":true,"scene":"本人主动授权"}),
        "consent",
        100,
    )
    .unwrap();
    messages::delivery::reserve(&mut s, &id, "attempt1", 100).unwrap();
    messages::delivery::finish(
        &mut s,
        &id,
        "attempt1",
        DeliveryOutcome::TemporaryFailure,
        100,
    )
    .unwrap();
    assert!(messages::delivery::reserve(&mut s, &id, "too-early", 109).is_err());
    messages::delivery::reserve(&mut s, &id, "attempt2", 110).unwrap();
    let r = messages::delivery::finish(
        &mut s,
        &id,
        "attempt2",
        DeliveryOutcome::TemporaryFailure,
        110,
    )
    .unwrap();
    assert_eq!(r["external_status"], "retry_exhausted");
    assert!(messages::delivery::reserve(&mut s, &id, "attempt3", 120).is_err());
    assert!(s.commissions.entries.is_empty());
    assert!(
        !messages::read(&s, "alice", "messages", "alice", 120)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}
#[test]
fn manual_risk_association_cannot_mass_ban_and_partial_holds_require_financial_permissions() {
    let mut s = setup();
    task(&mut s, "task", true);
    risk::command(&mut s,"risk","risk_rule_publish",&json!({"id":"association","version":0,"signal":"account_association","enabled":true,"starts_at":0,"ends_at":1000,"purpose":"必要业务核查","evaluation":"不能依据单一相同网络","reason":"测试"}),"x",100).unwrap();
    risk::command(&mut s,"risk","risk_case_open",&json!({"owner":"alice","rule_id":"association","business_id":"task","evidence_summary":"已授权业务关联信号","authorized_signals":true,"reason":"人工核查"}),"case",101).unwrap();
    assert!(risk::command(&mut s,"risk","risk_case_decide",&json!({"id":"case","version":1,"decision":"restrict","scope":"task_claim","expires_at":200,"reason":"单一信号不可封禁"}),"x",102).is_err());
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1000
    );
    risk::command(&mut s,"risk","risk_rule_publish",&json!({"id":"evidence-rule","version":0,"signal":"duplicate_evidence","enabled":true,"starts_at":0,"ends_at":1000,"purpose":"合法成果核查","evaluation":"需要独立处理","reason":"测试"}),"x",100).unwrap();
    risk::command(&mut s,"risk","risk_case_open",&json!({"owner":"alice","rule_id":"evidence-rule","business_id":"task","evidence_summary":"脱敏证据核查信号","authorized_signals":true,"reason":"人工核查"}),"hold-case",101).unwrap();
    let input = json!({"id":"hold-case","version":1,"decision":"hold_source","source":"task","amount_cents":300,"reason":"仅暂缓该任务来源"});
    assert!(risk::command(&mut s, "risk", "risk_case_decide", &input, "x", 102).is_err());
    s.users
        .get_mut("risk")
        .unwrap()
        .roles
        .push("settlement_reviewer".into());
    risk::command(&mut s, "risk", "risk_case_decide", &input, "x", 102).unwrap();
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        700
    );
    let release =
        json!({"id":"hold-case","version":2,"decision":"release","reason":"人工确认误判"});
    assert!(risk::command(&mut s, "risk", "risk_case_decide", &release, "x", 103).is_err());
    s.users
        .get_mut("risk")
        .unwrap()
        .roles
        .push("verifier".into());
    risk::command(&mut s, "risk", "risk_case_decide", &release, "x", 103).unwrap();
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1000
    );
}
