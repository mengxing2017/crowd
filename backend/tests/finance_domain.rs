use crowd_api::{
    commissions::domain as ledger, platform::domain::State, settlements::domain as settlements,
    withdrawals::domain as withdrawals,
};
use serde_json::{Value, json};
fn setup() -> State {
    let mut s = State::default();
    for id in [
        "alice", "bob", "manager", "reviewer", "handler", "other", "verifier", "second",
    ] {
        s.register_user(id, json!({}), None);
    }
    for (id, roles) in [
        ("manager", vec!["rule_manager"]),
        ("reviewer", vec!["settlement_reviewer"]),
        ("handler", vec!["manual_handler"]),
        ("other", vec!["manual_handler"]),
        ("verifier", vec!["verifier"]),
        ("second", vec!["verifier"]),
    ] {
        let u = s.users.get_mut(id).unwrap();
        u.roles = roles.into_iter().map(String::from).collect();
        u.scope = vec!["*".into()];
    }
    s.users.get_mut("alice").unwrap().receipts.push(json!({"id":"receipt","owner":"alice","version":2,"method":"wechat","file_id":"private-original","status":"confirmed","official_verified":false}));
    ledger::append(
        &mut s,
        "seed",
        "alice",
        "confirmed",
        1000,
        "task",
        "reviewer",
        100,
        json!({"calculation":{"amount_cents":1000},"price":{"version":1}}),
    )
    .unwrap();
    let rule = json!({"min_cents":100,"max_cents":1000,"daily_count":10,"daily_cents":10000,"lifetime_cents":100000,"second_review_cents":0,"reminder_seconds":60,"registration_verification":true,"separate_roles":true,"methods":["wechat"],"reason":"仅隔离测试规则，非正式业务门槛"});
    withdrawals::command(
        &mut s,
        "manager",
        "withdrawal_rule_propose",
        &rule,
        "rule",
        100,
    )
    .unwrap();
    withdrawals::command(
        &mut s,
        "verifier",
        "withdrawal_rule_review",
        &json!({"id":"rule","approve":true,"reason":"独立配置复核"}),
        "publish",
        101,
    )
    .unwrap();
    s
}
fn apply(s: &mut State, id: &str, n: i64) -> Value {
    let body = json!({"amount_cents":n,"receipt_id":"receipt","receipt_version":2,"summary_version":ledger::summary(s,"alice").unwrap()["version"]});
    let p = withdrawals::command(
        s,
        "alice",
        "withdrawal_preview",
        &body,
        &format!("preview-{id}"),
        102,
    )
    .unwrap();
    let mut b = body;
    b["preview_id"] = p["id"].clone();
    b["confirmed"] = json!(true);
    withdrawals::command(s, "alice", "withdrawal_apply", &b, id, 103).unwrap()
}
fn process(
    s: &mut State,
    actor: &str,
    a: &str,
    id: &str,
    mut b: Value,
) -> Result<Value, crowd_api::shared::Failure> {
    b["id"] = json!(id);
    b["version"] = s.withdrawals.applications[id]["version"].clone();
    if b["reason"].is_null() {
        b["reason"] = json!("测试核对说明");
    }
    let old = s.clone();
    let r = settlements::command(s, actor, a, &b, &format!("{a}-{id}"), 200);
    if r.is_err() {
        *s = old;
    }
    r
}
fn ready(s: &mut State, id: &str) {
    process(
        s,
        "reviewer",
        "settlement_review",
        id,
        json!({"approve":true,"receipt_checked":true,"source_checked":true}),
    )
    .unwrap();
}
#[test]
fn integer_summary_and_serialization_match_platform_contract() {
    let s = setup();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../api-contract/fixtures/commission-summary.json"
    ))
    .unwrap();
    assert_eq!(ledger::summary(&s, "alice").unwrap(), fixture);
    for v in [
        json!(-1),
        json!(1.5),
        json!(0),
        json!(9007199254740992_i64),
        json!(i64::MIN),
    ] {
        let mut s = setup();
        assert!(withdrawals::command(&mut s,"alice","withdrawal_preview",&json!({"amount_cents":v,"receipt_id":"receipt","receipt_version":2,"summary_version":1}),"p",102).is_err());
    }
}
#[test]
fn application_holds_once_keeps_receipt_snapshot_and_source_allocations() {
    let mut s = setup();
    let app = apply(&mut s, "a", 600);
    assert!(app["receipt_snapshot"]["file_id"].is_null());
    let sum = ledger::summary(&s, "alice").unwrap();
    assert_eq!(sum["available_cents"], 400);
    assert_eq!(sum["held_cents"], 600);
    assert_eq!(sum["consistent"], true);
    assert_eq!(ledger::sources(&s, "alice").unwrap()["task"], 400);
    s.users
        .get_mut("alice")
        .unwrap()
        .receipts
        .push(json!({"id":"new","method":"wechat","version":3,"file_id":"new-file"}));
    assert_eq!(
        s.withdrawals.applications["a"]["receipt_snapshot"]["file_id"],
        "private-original"
    );
    assert!(
        withdrawals::command(
            &mut s,
            "bob",
            "withdrawal_receipt_reveal",
            &json!({"id":"a","reason":"攻击"}),
            "x",
            104
        )
        .is_err()
    );
    assert!(
        withdrawals::command(
            &mut s,
            "handler",
            "withdrawal_receipt_reveal",
            &json!({"id":"a","reason":"未领取"}),
            "x",
            104
        )
        .is_err()
    );
}
#[test]
fn cancellation_and_rejection_release_exactly_once() {
    let mut s = setup();
    apply(&mut s, "a", 600);
    let b = json!({"id":"a","version":1,"reason":"用户主动撤销"});
    withdrawals::command(&mut s, "alice", "withdrawal_cancel", &b, "cancel", 105).unwrap();
    assert!(
        withdrawals::command(
            &mut s,
            "alice",
            "withdrawal_cancel",
            &b,
            "cancel-again",
            105
        )
        .is_err()
    );
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1000
    );
    apply(&mut s, "b", 600);
    process(
        &mut s,
        "reviewer",
        "settlement_review",
        "b",
        json!({"approve":false}),
    )
    .unwrap();
    assert!(
        process(
            &mut s,
            "reviewer",
            "settlement_review",
            "b",
            json!({"approve":false})
        )
        .is_err()
    );
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1000
    );
}
#[test]
fn handling_unknown_never_releases_and_requires_confirmed_unpaid_stop() {
    let mut s = setup();
    apply(&mut s, "a", 600);
    ready(&mut s, "a");
    assert_eq!(s.withdrawals.applications["a"]["status"], "ready");
    assert_eq!(ledger::summary(&s, "alice").unwrap()["settled_cents"], 0);
    process(&mut s, "handler", "settlement_claim", "a", json!({})).unwrap();
    assert!(process(&mut s, "other", "settlement_claim", "a", json!({})).is_err());
    assert!(
        withdrawals::command(
            &mut s,
            "alice",
            "withdrawal_cancel",
            &json!({"id":"a","version":3,"reason":"取消"}),
            "c",
            201
        )
        .is_err()
    );
    process(&mut s, "handler", "settlement_unknown", "a", json!({})).unwrap();
    assert_eq!(ledger::summary(&s, "alice").unwrap()["held_cents"], 600);
    assert!(
        process(
            &mut s,
            "handler",
            "settlement_confirm_unpaid",
            "a",
            json!({"confirmed_unpaid":true})
        )
        .is_err()
    );
    process(
        &mut s,
        "handler",
        "settlement_confirm_unpaid",
        "a",
        json!({"confirmed_unpaid":true,"processing_stopped":true}),
    )
    .unwrap();
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1000
    );
}
#[test]
fn manual_registration_independent_verification_and_post_settlement_corrections() {
    let mut s = setup();
    apply(&mut s, "a", 600);
    ready(&mut s, "a");
    process(&mut s, "handler", "settlement_claim", "a", json!({})).unwrap();
    let body = json!({"amount_cents":600,"paid_at":150,"receipt_id":"receipt","receipt_confirmed":true,"voucher_file_id":"voucher","voucher_matches":true});
    process(&mut s, "handler", "settlement_register", "a", body.clone()).unwrap();
    assert_eq!(
        s.withdrawals.applications["a"]["status"],
        "awaiting_verification"
    );
    assert_eq!(ledger::summary(&s, "alice").unwrap()["held_cents"], 600);
    assert!(process(&mut s, "handler", "settlement_register", "a", body).is_err());
    assert!(
        process(
            &mut s,
            "handler",
            "settlement_verify",
            "a",
            json!({"approve":true,"voucher_checked":true})
        )
        .is_err()
    );
    process(
        &mut s,
        "verifier",
        "settlement_verify",
        "a",
        json!({"approve":true,"voucher_checked":true}),
    )
    .unwrap();
    let sum = ledger::summary(&s, "alice").unwrap();
    assert_eq!(sum["settled_cents"], 600);
    assert_eq!(sum["held_cents"], 0);
    assert_eq!(sum["available_cents"], 400);
    assert_eq!(sum["consistent"], true);
    assert_eq!(ledger::sources(&s, "alice").unwrap()["task"], 400);
    assert!(
        process(
            &mut s,
            "verifier",
            "settlement_verify",
            "a",
            json!({"approve":true,"voucher_checked":true})
        )
        .is_err()
    );
    process(
        &mut s,
        "handler",
        "settlement_correction_propose",
        "a",
        json!({"confirmed_unpaid":false,"evidence":"原登记发现差异，线下核查"}),
    )
    .unwrap();
    process(
        &mut s,
        "second",
        "settlement_correction_review",
        "a",
        json!({"approve":true}),
    )
    .unwrap();
    assert_eq!(
        s.withdrawals.applications["a"]["post_settlement_dispute"],
        true
    );
    assert_eq!(ledger::summary(&s, "alice").unwrap()["settled_cents"], 600);
}
#[test]
fn high_value_review_and_role_separation_cannot_be_bypassed() {
    let mut s = setup();
    s.withdrawals.rules[0]["second_review_cents"] = json!(500);
    apply(&mut s, "a", 600);
    ready(&mut s, "a");
    assert_eq!(
        s.withdrawals.applications["a"]["status"],
        "awaiting_second_review"
    );
    assert!(process(&mut s, "handler", "settlement_claim", "a", json!({})).is_err());
    s.users
        .get_mut("reviewer")
        .unwrap()
        .roles
        .push("verifier".into());
    assert!(
        process(
            &mut s,
            "reviewer",
            "settlement_second_review",
            "a",
            json!({"approve":true,"receipt_checked":true,"source_checked":true})
        )
        .is_err()
    );
    process(
        &mut s,
        "verifier",
        "settlement_second_review",
        "a",
        json!({"approve":true,"receipt_checked":true,"source_checked":true}),
    )
    .unwrap();
    s.users
        .get_mut("reviewer")
        .unwrap()
        .roles
        .push("manual_handler".into());
    assert!(process(&mut s, "reviewer", "settlement_claim", "a", json!({})).is_err());
}
#[test]
fn disputes_only_affect_their_sources_and_adjustments_need_independent_approval() {
    let mut s = setup();
    ledger::append(
        &mut s,
        "other-source",
        "alice",
        "increase",
        200,
        "other-task",
        "reviewer",
        100,
        json!({}),
    )
    .unwrap();
    ledger::command(
        &mut s,
        "reviewer",
        "commission_dispute",
        &json!({"owner":"alice","source":"task","amount_cents":300,"reason":"仅争议此来源"}),
        "d",
        104,
    )
    .unwrap();
    apply(&mut s, "a", 600);
    ledger::command(
        &mut s,
        "verifier",
        "commission_dispute_resolve",
        &json!({"id":"d","confirmed":true,"reason":"已核查"}),
        "r",
        105,
    )
    .unwrap();
    assert_eq!(ledger::summary(&s, "alice").unwrap()["held_cents"], 600);
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        600
    );
    let b = json!({"owner":"alice","amount_cents":-200,"new_owner":"bob","source":"task","business_key":"fix-ownership","reason":"归属纠错审批依据"});
    ledger::command(
        &mut s,
        "reviewer",
        "commission_adjust_propose",
        &b,
        "fix",
        106,
    )
    .unwrap();
    assert!(
        ledger::command(
            &mut s,
            "reviewer",
            "commission_adjust_review",
            &json!({"id":"fix","approve":true,"reason":"自审"}),
            "x",
            107
        )
        .is_err()
    );
    ledger::command(
        &mut s,
        "verifier",
        "commission_adjust_review",
        &json!({"id":"fix","approve":true,"reason":"独立审批"}),
        "x",
        107,
    )
    .unwrap();
    assert_eq!(ledger::summary(&s, "bob").unwrap()["available_cents"], 200);
    assert!(
        ledger::command(
            &mut s,
            "reviewer",
            "commission_adjust_propose",
            &b,
            "fix2",
            108
        )
        .is_err()
    );
}
#[test]
fn receipt_mismatch_partial_registration_and_handoff_keep_holds() {
    let mut s = setup();
    apply(&mut s, "a", 600);
    ready(&mut s, "a");
    process(&mut s, "handler", "settlement_claim", "a", json!({})).unwrap();
    process(
        &mut s,
        "handler",
        "settlement_handoff",
        "a",
        json!({"to":"other","paid_status":"unknown"}),
    )
    .unwrap();
    assert!(process(&mut s,"handler","settlement_register","a",json!({"amount_cents":600,"paid_at":150,"receipt_id":"receipt","receipt_confirmed":true,"voucher_file_id":"v","voucher_matches":true})).is_err());
    process(&mut s, "other", "settlement_handoff_accept", "a", json!({})).unwrap();
    process(&mut s,"other","settlement_register","a",json!({"amount_cents":300,"paid_at":150,"receipt_id":"receipt","receipt_confirmed":true,"voucher_file_id":"v","voucher_matches":true})).unwrap();
    assert_eq!(s.withdrawals.applications["a"]["status"], "unknown");
    assert_eq!(ledger::summary(&s, "alice").unwrap()["held_cents"], 600);
    assert!(
        process(
            &mut s,
            "verifier",
            "settlement_verify",
            "a",
            json!({"approve":true,"voucher_checked":true})
        )
        .is_err()
    );
}
#[test]
fn absent_rule_draft_alipay_restrictions_and_data_scope_fail_closed() {
    let mut s = setup();
    s.withdrawals.rules.clear();
    assert!(
        withdrawals::command(&mut s, "alice", "withdrawal_preview", &json!({}), "p", 105).is_err()
    );
    let mut s = setup();
    s.users
        .get_mut("alice")
        .unwrap()
        .restrictions
        .push(json!({"scope":"withdrawal"}));
    assert!(withdrawals::command(&mut s,"alice","withdrawal_preview",&json!({"amount_cents":600,"receipt_id":"receipt","receipt_version":2,"summary_version":1}),"p",105).is_err());
    s.users.get_mut("reviewer").unwrap().scope = vec!["bob".into()];
    assert!(ledger::read(&s, "reviewer", "commission_summary", "alice").is_err());
    assert!(ledger::read(&s, "bob", "commission_entries", "alice|0").is_err());
}

#[test]
fn append_only_cursor_does_not_duplicate_entries_when_new_changes_arrive() {
    let mut s = setup();
    for n in 0..60 {
        ledger::append(
            &mut s,
            &format!("e-{n}"),
            "alice",
            "increase",
            1,
            "page-source",
            "reviewer",
            100 + n,
            json!({}),
        )
        .unwrap();
    }
    let page = ledger::read(&s, "alice", "commission_entries", "alice|0").unwrap();
    let cursor = page["next_cursor"].as_u64().unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 50);
    ledger::append(
        &mut s,
        "newer",
        "alice",
        "increase",
        1,
        "page-source",
        "reviewer",
        200,
        json!({}),
    )
    .unwrap();
    let second = ledger::read(
        &s,
        "alice",
        "commission_entries",
        &format!("alice|{cursor}"),
    )
    .unwrap();
    assert_eq!(second["items"].as_array().unwrap().len(), 11);
    for e in second["items"].as_array().unwrap() {
        assert!(
            !page["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|old| old["id"] == e["id"])
        );
    }
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1061
    );
}
#[test]
fn settled_disputes_keep_old_registration_without_future_deductions() {
    let mut s = setup();
    apply(&mut s, "a", 1000);
    ready(&mut s, "a");
    process(&mut s, "handler", "settlement_claim", "a", json!({})).unwrap();
    process(&mut s,"handler","settlement_register","a",json!({"amount_cents":1000,"paid_at":150,"receipt_id":"receipt","receipt_confirmed":true,"voucher_file_id":"v","voucher_matches":true})).unwrap();
    process(
        &mut s,
        "verifier",
        "settlement_verify",
        "a",
        json!({"approve":true,"voucher_checked":true}),
    )
    .unwrap();
    ledger::command(&mut s,"reviewer","commission_dispute",&json!({"owner":"alice","source":"task","amount_cents":1000,"reason":"已结算后来源失效，进入核查"}),"paid-dispute",205).unwrap();
    ledger::append(
        &mut s,
        "future",
        "alice",
        "increase",
        200,
        "new-task",
        "reviewer",
        206,
        json!({}),
    )
    .unwrap();
    let sum = ledger::summary(&s, "alice").unwrap();
    assert_eq!(sum["settled_cents"], 1000);
    assert_eq!(sum["available_cents"], 200);
    assert_eq!(
        s.commissions.disputes["paid-dispute"]["claimed_cents"],
        1000
    );
    assert_eq!(s.commissions.disputes["paid-dispute"]["amount_cents"], 0);
}
#[test]
fn orphan_holds_and_stale_versions_never_produce_partial_application() {
    let mut s = setup();
    ledger::append(
        &mut s,
        "orphan",
        "alice",
        "hold",
        300,
        "missing-application",
        "reviewer",
        100,
        json!({"sources":[{"source":"task","amount_cents":300}]}),
    )
    .unwrap();
    assert_eq!(ledger::summary(&s, "alice").unwrap()["consistent"], false);
    let b =
        json!({"amount_cents":100,"receipt_id":"receipt","receipt_version":2,"summary_version":2});
    assert!(withdrawals::command(&mut s, "alice", "withdrawal_preview", &b, "p", 105).is_err());
    assert!(s.withdrawals.applications.is_empty());
}

#[test]
fn pending_rejected_and_confirmed_task_digits_reconcile_only_once() {
    let mut s = setup();
    s.commissions.entries.clear();
    s.tasks.instances.insert("claim".into(),json!({"id":"claim","owner":"alice","reserved_cents":1000,"status":"submitted","submissions":[{"id":"submission"}],"price":{"kind":"fixed","amount_cents":1000,"version":1}}));
    ledger::sync_tasks(&mut s, "reviewer", 100).unwrap();
    let sum = ledger::summary(&s, "alice").unwrap();
    assert_eq!(sum["pending_cents"], 1000);
    assert_eq!(sum["available_cents"], 0);
    s.tasks.instances.get_mut("claim").unwrap()["status"] = json!("rejected");
    ledger::sync_tasks(&mut s, "reviewer", 101).unwrap();
    assert_eq!(ledger::summary(&s, "alice").unwrap()["available_cents"], 0);
    assert_eq!(ledger::summary(&s, "alice").unwrap()["pending_cents"], 0);
    s.tasks.instances.get_mut("claim").unwrap()["status"] = json!("approved");
    s.tasks.confirmations.insert("claim".into(),json!({"id":"claim","claim_id":"claim","owner":"alice","status":"pending_ledger","withdrawable":false,"calculation":{"amount_cents":1000},"price":{"kind":"fixed","amount_cents":1000,"version":1}}));
    ledger::sync_tasks(&mut s, "reviewer", 102).unwrap();
    ledger::sync_tasks(&mut s, "reviewer", 103).unwrap();
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1000
    );
    assert_eq!(
        s.commissions
            .entries
            .iter()
            .filter(|e| e["kind"] == "confirmed")
            .count(),
        1
    );
    assert_eq!(s.tasks.confirmations["claim"]["status"], "pending_ledger");
    s.tasks.instances.get_mut("claim").unwrap()["status"] = json!("disputed");
    ledger::sync_tasks(&mut s, "reviewer", 104).unwrap();
    assert_eq!(ledger::summary(&s, "alice").unwrap()["available_cents"], 0);
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["disputed_cents"],
        1000
    );
}
#[test]
fn source_navigation_only_shows_authorized_owner_and_fixed_application_versions() {
    let mut s = setup();
    apply(&mut s, "a", 600);
    let source = ledger::read(&s, "reviewer", "commission_source", "alice|task").unwrap();
    assert_eq!(source["applications"][0]["id"], "a");
    assert_eq!(source["applications"][0]["receipt_version"], 2);
    assert!(source["applications"][0]["receipt_snapshot"].is_null());
    assert!(ledger::read(&s, "bob", "commission_source", "alice|task").is_err());
}

#[test]
fn cancelled_application_with_disputed_source_moves_hold_to_dispute_instead_of_free_digits() {
    let mut s = setup();
    apply(&mut s, "a", 1000);
    ledger::command(&mut s,"reviewer","commission_dispute",&json!({"owner":"alice","source":"task","amount_cents":1000,"reason":"已有申请占用的来源需核查"}),"held-dispute",104).unwrap();
    assert_eq!(s.commissions.disputes["held-dispute"]["amount_cents"], 0);
    assert!(ledger::source_disputed(
        &s,
        &s.withdrawals.applications["a"]
    ));
    withdrawals::command(
        &mut s,
        "alice",
        "withdrawal_cancel",
        &json!({"id":"a","version":1,"reason":"停止本次申请"}),
        "c",
        105,
    )
    .unwrap();
    let sum = ledger::summary(&s, "alice").unwrap();
    assert_eq!(sum["held_cents"], 0);
    assert_eq!(sum["available_cents"], 0);
    assert_eq!(sum["disputed_cents"], 1000);
    assert_eq!(sum["consistent"], true);
    ledger::sync_tasks(&mut s, "reviewer", 106).unwrap();
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["disputed_cents"],
        1000
    );
    ledger::command(
        &mut s,
        "verifier",
        "commission_dispute_resolve",
        &json!({"id":"held-dispute","reason":"独立核查后解除","confirmed":true}),
        "r",
        107,
    )
    .unwrap();
    assert_eq!(
        ledger::summary(&s, "alice").unwrap()["available_cents"],
        1000
    );
}
