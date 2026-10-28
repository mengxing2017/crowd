use super::task_support as support;
use crowd_api::{
    community::domain::State,
    tasks::{south::port::VerifiedTaskResult, south::port::rules::calculate},
};
use serde_json::{Value, json};
fn setup() -> State {
    let mut s = State::default();
    for id in ["editor", "verifier", "reviewer", "second", "alice", "bob"] {
        s.register(id, &format!("c-{id}"), json!({}), None).unwrap();
    }
    for (id, roles) in [
        ("editor", vec!["operator", "rule_manager"]),
        ("verifier", vec!["verifier"]),
        ("reviewer", vec!["task_reviewer"]),
        ("second", vec!["task_reviewer", "verifier"]),
    ] {
        let u = s.users.get_mut(id).unwrap();
        u.roles = roles.into_iter().map(String::from).collect();
        u.scope = vec!["*".into()];
    }
    support::seed(&mut s, "editor", "verifier", 100);
    support::instance(&mut s, "a", "alice", 100);
    s
}
fn cmd(s: &mut State, actor: &str, a: &str, body: Value, now: i64) -> Value {
    let mut b = body;
    let id = b["id"].as_str().unwrap_or("a").to_owned();
    b["id"] = json!(id);
    b["version"] = s.tasks.instances[&id]["version"].clone();
    s.tasks_command(actor, a, &b, &format!("{a}-{now}"), now)
        .unwrap()
}
fn submitted(s: &mut State) {
    cmd(s, "alice", "task_start", json!({}), 110);
    cmd(s, "alice", "task_step", json!({"step_id":"one"}), 111);
    cmd(s, "alice", "task_step", json!({"step_id":"two"}), 112);
    cmd(
        s,
        "alice",
        "evidence_save",
        json!({"evidence":{"fields":{"description":"真实业务说明","order":" Order-123 "},"occurred_at":115}}),
        120,
    );
    let p = cmd(s, "alice", "evidence_preview", json!({}), 121);
    cmd(
        s,
        "alice",
        "evidence_submit",
        json!({"preview_id":p["preview"]["id"],"confirmed":true}),
        122,
    );
}
#[test]
fn integral_pricing_rounding_cap_and_overflow() {
    assert_eq!(
        calculate(&json!({"kind":"fixed","amount_cents":101}), None).unwrap()["amount_cents"],
        101
    );
    for (round, amount) in [("floor", 33), ("ceil", 34), ("half_up", 34)] {
        assert_eq!(
            calculate(
                &json!({"kind":"proportional","numerator":1,"denominator":3,"rounding":round}),
                Some(101)
            )
            .unwrap()["amount_cents"],
            amount
        );
    }
    assert_eq!(calculate(&json!({"kind":"proportional","numerator":1,"denominator":1,"rounding":"floor","cap_cents":10}),Some(100)).unwrap()["amount_cents"],10);
    assert!(calculate(&json!({"kind":"fixed","amount_cents":-1}), None).is_err());
    assert!(
        calculate(
            &json!({"kind":"proportional","numerator":1,"denominator":0,"rounding":"floor"}),
            Some(1)
        )
        .is_err()
    );
    assert!(
        calculate(
            &json!({"kind":"fixed","amount_cents":9007199254740992_i64}),
            None
        )
        .is_err()
    );
}
#[test]
fn independent_rule_review_overlap_and_immutable_versions() {
    let mut s = setup();
    s.users
        .get_mut("editor")
        .unwrap()
        .roles
        .push("verifier".into());
    let p = support::price(100);
    s.tasks_command(
        "editor",
        "price_save",
        &json!({"id":"new","content":p}),
        "x",
        101,
    )
    .unwrap();
    assert!(
        s.tasks_command(
            "editor",
            "price_review",
            &json!({"id":"new","version":1,"approve":true,"reason":"自审"}),
            "x",
            102
        )
        .is_err()
    );
    assert_eq!(
        s.tasks_command(
            "verifier",
            "price_review",
            &json!({"id":"new","version":1,"approve":true,"reason":"冲突"}),
            "x",
            102
        )
        .unwrap_err()
        .0,
        "RULE_OVERLAP"
    );
    assert!(
        s.tasks_command(
            "editor",
            "price_save",
            &json!({"id":"price","version":2,"content":p}),
            "x",
            102
        )
        .is_err()
    );
}
#[test]
fn ordered_steps_drafts_preview_and_immutable_receipt() {
    let mut s = setup();
    cmd(&mut s, "alice", "task_start", json!({}), 110);
    assert!(
        s.tasks_command(
            "alice",
            "task_step",
            &json!({"id":"a","version":2,"step_id":"two"}),
            "x",
            111
        )
        .is_err()
    );
    assert!(
        s.tasks_command(
            "bob",
            "task_start",
            &json!({"id":"a","version":2}),
            "x",
            111
        )
        .is_err()
    );
    cmd(&mut s, "alice", "task_step", json!({"step_id":"one"}), 111);
    cmd(&mut s, "alice", "task_step", json!({"step_id":"two"}), 112);
    cmd(
        &mut s,
        "alice",
        "evidence_save",
        json!({"evidence":{"fields":{}}}),
        113,
    );
    assert!(s.tasks.confirmations.is_empty());
    assert!(
        s.tasks_command(
            "alice",
            "evidence_preview",
            &json!({"id":"a","version":s.tasks.instances["a"]["version"]}),
            "x",
            114
        )
        .is_err()
    );
    cmd(
        &mut s,
        "alice",
        "evidence_save",
        json!({"evidence":{"fields":{"description":"完成"},"occurred_at":115}}),
        116,
    );
    let p = cmd(&mut s, "alice", "evidence_preview", json!({}), 117);
    cmd(
        &mut s,
        "alice",
        "evidence_submit",
        json!({"confirmed":true,"preview_id":p["preview"]["id"]}),
        118,
    );
    let before = s.tasks.instances["a"]["submissions"].clone();
    assert!(
        s.tasks_command(
            "alice",
            "evidence_save",
            &json!({"id":"a","version":s.tasks.instances["a"]["version"],"evidence":{}}),
            "x",
            119
        )
        .is_err()
    );
    assert_eq!(s.tasks.instances["a"]["submissions"], before);
}
#[test]
fn exclusive_review_withdrawal_sensitive_access_and_single_confirmation() {
    let mut s = setup();
    submitted(&mut s);
    let queue = s.tasks_read("reviewer", "task_queue", "{}", 123).unwrap();
    assert!(queue["items"][0]["draft"].is_null());
    assert!(queue["items"][0]["receipt"]["evidence"].is_null());
    cmd(&mut s, "reviewer", "review_claim", json!({}), 123);
    let version = s.tasks.instances["a"]["version"].clone();
    assert!(
        s.tasks_command(
            "second",
            "review_claim",
            &json!({"id":"a","version":version}),
            "x",
            124
        )
        .is_err()
    );
    assert!(
        s.tasks_command(
            "alice",
            "evidence_withdraw",
            &json!({"id":"a","version":version,"reason":"撤回"}),
            "x",
            124
        )
        .is_err()
    );
    assert!(
        s.tasks_command(
            "second",
            "evidence_reveal",
            &json!({"id":"a","version":version,"reason":"窥探"}),
            "x",
            124
        )
        .is_ok()
    ); // independent verifier may inspect with recorded purpose
    let result = cmd(
        &mut s,
        "reviewer",
        "review_decide",
        json!({"decision":"approve","reason":"核对通过","checklist_confirmed":true}),
        125,
    );
    assert_eq!(result["status"], "approved");
    assert_eq!(s.tasks.confirmations.len(), 1);
    assert_eq!(s.tasks.changes.len(), 1);
    assert_eq!(s.tasks.confirmations["a"]["withdrawable"], true);
    assert!(s.tasks_command("reviewer","review_decide",&json!({"id":"a","version":result["version"],"decision":"approve","reason":"重复","checklist_confirmed":true}),"x",126).is_err());
}
#[test]
fn supplement_new_version_preserves_old_and_only_requested_fields() {
    let mut s = setup();
    submitted(&mut s);
    let old = s.tasks.instances["a"]["submissions"][0].clone();
    cmd(&mut s, "reviewer", "review_claim", json!({}), 123);
    cmd(
        &mut s,
        "reviewer",
        "review_decide",
        json!({"decision":"supplement","reason":"需完整说明","fields":["description"]}),
        124,
    );
    cmd(
        &mut s,
        "alice",
        "evidence_save",
        json!({"evidence":{"fields":{"description":"补充完整说明","order":" Order-123 "},"occurred_at":115}}),
        125,
    );
    let p = cmd(&mut s, "alice", "evidence_preview", json!({}), 126);
    let t = cmd(
        &mut s,
        "alice",
        "evidence_submit",
        json!({"preview_id":p["preview"]["id"],"confirmed":true}),
        127,
    );
    assert_eq!(t["submissions"][0], old);
    assert_eq!(t["receipt"]["version"], 2);
    assert!(s.tasks.confirmations.is_empty());
}
#[test]
fn pause_extends_once_timeout_releases_once_and_never_times_out_review() {
    let mut s = setup();
    cmd(
        &mut s,
        "editor",
        "task_pause",
        json!({"reason":"业务暂停"}),
        110,
    );
    cmd(
        &mut s,
        "editor",
        "task_resume",
        json!({"reason":"恢复"}),
        210,
    );
    assert_eq!(s.tasks.instances["a"]["submit_until"], 2200);
    assert_eq!(s.expire_tasks(2200), 0);
    assert_eq!(s.expire_tasks(2201), 1);
    assert_eq!(s.expire_tasks(2300), 0);
    assert_eq!(s.catalog.claims["a"]["reservation_released"], true);
    let mut s = setup();
    submitted(&mut s);
    assert_eq!(s.expire_tasks(99999), 0);
    assert_eq!(s.tasks.instances["a"]["status"], "submitted");
}
#[test]
fn high_risk_independent_verification_and_dispute_append_only() {
    let mut s = setup();
    s.tasks.instances.get_mut("a").unwrap()["template"]["double_review"] = json!(true);
    submitted(&mut s);
    cmd(&mut s, "reviewer", "review_claim", json!({}), 123);
    cmd(
        &mut s,
        "reviewer",
        "review_decide",
        json!({"decision":"approve","reason":"初审通过","checklist_confirmed":true}),
        124,
    );
    assert!(s.tasks.confirmations.is_empty());
    cmd(
        &mut s,
        "verifier",
        "review_verify",
        json!({"approve":true,"reason":"复核通过","checklist_confirmed":true}),
        125,
    );
    let old = s.tasks.confirmations["a"].clone();
    let proposal = cmd(
        &mut s,
        "reviewer",
        "review_revoke_propose",
        json!({"reason":"发现问题","evidence":"抽检证据"}),
        126,
    );
    cmd(
        &mut s,
        "verifier",
        "review_revoke_review",
        json!({"proposal_id":proposal["id"],"approve":true,"reason":"独立核查"}),
        127,
    );
    assert_eq!(s.tasks.confirmations["a"], old);
    assert_eq!(s.tasks.instances["a"]["status"], "disputed");
    assert_eq!(s.tasks.changes.len(), 2);
    assert_eq!(s.tasks.changes[1]["external_clawback"], false);
}
#[test]
fn trusted_result_port_deduplicates_source_and_calculates_ratio() {
    let mut s = setup();
    submitted(&mut s);
    let t = s.tasks.instances.get_mut("a").unwrap();
    t["template"]["source"] = json!("both");
    t["price"] = json!({"id":"ratio","version":2,"kind":"proportional","numerator":1,"denominator":3,"rounding":"half_up","cap_cents":100,"base_source":"signed-source"});
    cmd(&mut s, "reviewer", "review_claim", json!({}), 123);
    assert!(s.tasks_command("reviewer","review_decide",&json!({"id":"a","version":s.tasks.instances["a"]["version"],"decision":"approve","reason":"未获可信成果","checklist_confirmed":true}),"x",124).is_err());
    let first = s
        .record_task_result("signed-source", "event-1", "a", 115, Some(101))
        .unwrap();
    assert_eq!(
        s.record_task_result("signed-source", "event-1", "a", 115, Some(101))
            .unwrap(),
        first
    );
    assert!(
        s.record_task_result("signed-source", "event-1", "a", 115, Some(999))
            .is_err()
    );
    cmd(
        &mut s,
        "reviewer",
        "review_decide",
        json!({"decision":"approve","reason":"核对可信成果","checklist_confirmed":true}),
        125,
    );
    assert_eq!(
        s.tasks.confirmations["a"]["calculation"]["amount_cents"],
        34
    );
}
#[test]
fn eligibility_period_cap_disable_and_legacy_compatibility() {
    let mut s = setup();
    support::instance(&mut s, "b", "alice", 101);
    let task = json!({"id":"task","template_id":"template","price_id":"price","commission_cents":101,"currency":"CNY"});
    assert!(s.task_snapshot("alice", &task, 102).is_err());
    assert!(
        s.task_snapshot("alice", &json!({"id":"legacy"}), 102)
            .unwrap()
            .is_none()
    );
    let p = s.tasks.prices.get_mut("price").unwrap();
    p["content"]["period_cap_cents"] = json!(100);
    p["content"]["period_seconds"] = json!(86400);
    p["content"]["period_anchor"] = json!(0);
    assert!(s.task_snapshot("bob", &task, 102).is_err());
    s.tasks_command(
        "editor",
        "template_disable",
        &json!({"id":"template","version":2,"reason":"停止新领取"}),
        "x",
        103,
    )
    .unwrap();
    assert!(s.task_snapshot("bob", &task, 104).is_err());
    assert_eq!(s.tasks.instances["a"]["template"]["name"], "完成业务操作");
}

#[test]
fn order_normalization_duplicate_detection_and_private_file_ownership() {
    let mut s = setup();
    submitted(&mut s);
    support::instance(&mut s, "b", "bob", 101);
    cmd(&mut s, "bob", "task_start", json!({"id":"b"}), 125);
    assert!(s.tasks_command("bob","evidence_save",&json!({"id":"b","version":s.tasks.instances["b"]["version"],"evidence":{"fields":{"description":"另一任务","order":"order-123"},"occurred_at":126}}),"duplicate-order",127).is_err());
}
#[test]
fn failed_file_ownership_and_unassigned_sensitive_view_are_denied() {
    let mut s = setup();
    cmd(&mut s, "alice", "task_start", json!({}), 110);
    s.tasks
        .attachments
        .insert("foreign".into(), json!({"task_id":"other","owner":"bob"}));
    assert!(s.tasks_command("alice","evidence_save",&json!({"id":"a","version":s.tasks.instances["a"]["version"],"evidence":{"fields":{"photo":["foreign"]}}}),"x",111).is_err());
    let mut s = setup();
    submitted(&mut s);
    assert!(
        s.tasks_command(
            "reviewer",
            "evidence_reveal",
            &json!({"id":"a","version":s.tasks.instances["a"]["version"],"reason":"未领取"}),
            "x",
            123
        )
        .is_err()
    );
}
#[test]
fn reminders_are_once_per_submission_and_do_not_approve_or_release() {
    let mut s = setup();
    submitted(&mut s);
    cmd(&mut s, "reviewer", "review_claim", json!({}), 123);
    assert_eq!(s.expire_tasks(223), 0);
    let notifications = s.catalog.notifications.len();
    assert!(notifications > 0);
    assert_eq!(s.expire_tasks(224), 0);
    assert_eq!(s.catalog.notifications.len(), notifications);
    assert_eq!(s.tasks.instances["a"]["status"], "reviewing");
    assert!(
        !s.tasks.instances["a"]["reservation_released"]
            .as_bool()
            .unwrap()
    );
    assert!(s.tasks.confirmations.is_empty());
}
#[test]
fn repair_needs_independent_approval_and_cannot_bypass_review() {
    let mut s = setup();
    let p = cmd(
        &mut s,
        "editor",
        "task_repair_propose",
        json!({"new_status":"in_progress","reason":"状态需恢复","evidence":"事故工单"}),
        110,
    );
    let t = cmd(
        &mut s,
        "verifier",
        "task_repair_review",
        json!({"proposal_id":p["id"],"approve":true,"reason":"独立确认"}),
        111,
    );
    assert_eq!(t["status"], "in_progress");
    assert_eq!(t["started_at"], 111);
    assert!(s.tasks.confirmations.is_empty());
    assert!(s.tasks_command("editor","task_repair_propose",&json!({"id":"a","version":t["version"],"new_status":"approved","reason":"绕过","evidence":"无"}),"x",112).is_err());
}

#[test]
fn batch_preview_binds_exact_entries_versions_and_safe_integer_total() {
    let mut s = setup();
    submitted(&mut s);
    cmd(&mut s, "reviewer", "review_claim", json!({}), 123);
    let input = json!({"entries":[{"id":"a","version":s.tasks.instances["a"]["version"]}]});
    let preview = s
        .tasks_command(
            "reviewer",
            "review_batch_preview",
            &input,
            "batch-preview",
            124,
        )
        .unwrap();
    assert_eq!(preview["amount_cents"], 101);
    assert!(s.tasks_command("reviewer","review_batch",&json!({"entries":[{"id":"a","version":1}],"preview_id":"batch-preview","confirmed":true,"reason":"版本变化"}),"x",125).is_err());
    s.tasks_command("reviewer","review_batch",&json!({"entries":input["entries"],"preview_id":"batch-preview","confirmed":true,"reason":"已核对全部条目"}),"x",125).unwrap();
    assert_eq!(s.tasks.confirmations.len(), 1);
}

#[test]
fn withdraw_then_discard_keeps_immutable_submissions_and_receipt() {
    let mut s = setup();
    submitted(&mut s);
    let original = s.tasks.instances["a"]["submissions"].clone();
    cmd(
        &mut s,
        "alice",
        "evidence_withdraw",
        json!({"reason":"完善后重提"}),
        123,
    );
    let t = cmd(&mut s, "alice", "evidence_discard", json!({}), 124);
    assert!(t["draft"].is_null());
    assert_eq!(t["submissions"], original);
    assert_eq!(t["receipt"]["version"], 1);
}

#[test]
fn publication_checks_rule_display_and_freezes_preclaim_conditions() {
    let s = setup();
    let mut task = json!({"id":"task","template_id":"template","price_id":"price","completion":"按步骤提供真实说明","commission_cents":101,"submit_seconds":2000});
    s.validate_task_reference(&mut task, 100, 1000).unwrap();
    assert_eq!(task["price_preview"]["amount_cents"], 101);
    assert_eq!(
        task["template_preview"]["steps"].as_array().unwrap().len(),
        2
    );
    task["commission_cents"] = json!(102);
    assert!(s.validate_task_reference(&mut task, 100, 1000).is_err());
}

#[test]
fn proportional_cap_is_optional_but_unbounded_prices_cannot_reserve_a_numeric_limit() {
    let mut s = setup();
    let price = json!({"task_id":"task","settlement_scope":"unbounded","currency":"CNY","starts_at":0,"ends_at":10000,"kind":"proportional","numerator":1,"denominator":3,"rounding":"half_up","base_source":"source"});
    s.tasks_command(
        "editor",
        "price_save",
        &json!({"id":"unbounded","content":price}),
        "x",
        100,
    )
    .unwrap();
    s.tasks_command(
        "verifier",
        "price_review",
        &json!({"id":"unbounded","version":1,"approve":true,"reason":"不设置单笔封顶"}),
        "x",
        101,
    )
    .unwrap();
    let mut task = json!({"id":"task","template_id":"template","price_id":"unbounded","commission_cents":0,"currency":"CNY","completion":"按步骤提供真实说明","submit_seconds":2000});
    s.validate_task_reference(&mut task, 100, 1000).unwrap();
    assert!(task["price_preview"]["cap_cents"].is_null());
    assert_eq!(
        s.task_snapshot("bob", &task, 102).unwrap().unwrap()["reserved_cents"],
        0
    );
    task["digital_limit_cents"] = json!(100);
    assert!(s.validate_task_reference(&mut task, 100, 1000).is_err());
    let mut with_period = price.clone();
    with_period["period_cap_cents"] = json!(100);
    with_period["period_seconds"] = json!(1000);
    with_period["period_anchor"] = json!(0);
    with_period["reservation_policy"] = json!("reserve_max_release_terminal");
    assert!(
        s.tasks_command(
            "editor",
            "price_save",
            &json!({"id":"invalid-period","content":with_period}),
            "x",
            102
        )
        .is_err()
    );
}

#[test]
fn withdrawing_supplement_does_not_bypass_field_or_deadline_restrictions() {
    let mut s = setup();
    submitted(&mut s);
    cmd(&mut s, "reviewer", "review_claim", json!({}), 123);
    cmd(
        &mut s,
        "reviewer",
        "review_decide",
        json!({"decision":"supplement","reason":"补充说明","fields":["description"]}),
        124,
    );
    let p = cmd(&mut s, "alice", "evidence_preview", json!({}), 125);
    cmd(
        &mut s,
        "alice",
        "evidence_submit",
        json!({"preview_id":p["preview"]["id"],"confirmed":true}),
        126,
    );
    let t = cmd(
        &mut s,
        "alice",
        "evidence_withdraw",
        json!({"reason":"重新核对"}),
        127,
    );
    assert_eq!(t["status"], "supplement");
    cmd(
        &mut s,
        "alice",
        "evidence_save",
        json!({"evidence":{"fields":{"description":"补充","order":"CHANGED"},"occurred_at":115}}),
        128,
    );
    assert!(
        s.tasks_command(
            "alice",
            "evidence_preview",
            &json!({"id":"a","version":s.tasks.instances["a"]["version"]}),
            "x",
            129
        )
        .is_err()
    );
}

#[test]
fn verifier_only_role_can_queue_and_verify_without_initial_review_permission() {
    let mut s = setup();
    submitted(&mut s);
    assert!(
        s.tasks_read("verifier", "task_queue", "{}", 123).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    s.tasks.instances.get_mut("a").unwrap()["template"]["double_review"] = json!(true);
    cmd(&mut s, "reviewer", "review_claim", json!({}), 123);
    cmd(
        &mut s,
        "reviewer",
        "review_decide",
        json!({"decision":"approve","reason":"初审通过","checklist_confirmed":true}),
        124,
    );
    assert_eq!(
        s.tasks_read("verifier", "task_queue", "{}", 125).unwrap()["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    s.expire_tasks(300);
    assert!(
        s.catalog
            .notifications
            .iter()
            .any(|n| n["recipient"] == "verifier" && n["task_id"] == "a")
    );
    cmd(
        &mut s,
        "verifier",
        "review_verify",
        json!({"approve":true,"reason":"独立复核通过","checklist_confirmed":true}),
        301,
    );
    assert_eq!(s.tasks.confirmations.len(), 1);
}

#[test]
fn rust_task_rule_serialization_matches_the_cross_platform_contract_fixture() {
    let s = setup();
    let actual = s
        .tasks_read("editor", "task_rules", "platform", 100)
        .unwrap();
    let expected: Value =
        serde_json::from_str(include_str!("../../api-contract/fixtures/task-rules.json")).unwrap();
    assert_eq!(actual, expected);
}
