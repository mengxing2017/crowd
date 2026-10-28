use crowd_api::{
    platform::south::port::rules::State, withdrawals::south::port::rules as withdrawals,
};
use serde_json::{Value, json};
fn setup() -> State {
    let mut s = State::default();
    s.register_user("alice", json!({}), None);
    s.register_user("bob", json!({}), None);
    s
}
fn save(
    s: &mut State,
    method: &str,
    id: &str,
    account: &str,
) -> Result<Value, crowd_api::shared::Failure> {
    let v = s.users["alice"].version;
    s.command(
        "alice",
        "receipt",
        &json!({"method":method,"account":account,"file_id":id,"confirmed":true,"version":v}),
        id,
        "req",
        "2026-10-28T00:00:00Z",
    )
}
#[test]
fn alipay_draft_requires_new_confirmation_under_explicit_format_policy() {
    let mut s = setup();
    save(&mut s, "alipay", "draft", "alice@example.test").unwrap();
    assert_eq!(s.users["alice"].receipts[0]["status"], "draft");
    s.withdrawals.rules.push(json!({"id":"policy","status":"published","methods":["alipay"],"alipay_policy_confirmed":true,"alipay_formats":["email"],"alipay_name_required":false}));
    save(&mut s, "alipay", "confirmed", "alice@example.test").unwrap();
    let r = &s.users["alice"].receipts[1];
    assert_eq!(r["status"], "confirmed");
    assert_eq!(r["policy_id"], "policy");
    assert_eq!(r["declaration_version"], "receipt-self-v1");
    assert!(r["confirmed_at"].is_string());
    assert_eq!(s.users["alice"].receipts[0]["status"], "draft");
    for bad in ["bad", "x@@example.test", "x@host", "x@.host", "12345678901"] {
        assert!(save(&mut s, "alipay", "bad", bad).is_err());
    }
    s.withdrawals.rules[0]["alipay_name_required"] = json!(true);
    assert!(save(&mut s, "alipay", "name-required", "alice@example.test").is_err());
}
#[test]
fn draft_cannot_be_used_even_when_alipay_policy_was_later_enabled() {
    let mut s = setup();
    save(&mut s, "alipay", "draft", "alice@example.test").unwrap();
    s.withdrawals.rules.push(json!({"id":"policy","status":"published","methods":["alipay"],"alipay_policy_confirmed":true,"alipay_formats":["email"],"alipay_name_required":false}));
    let result = withdrawals::command(
        &mut s,
        "alice",
        "withdrawal_preview",
        &json!({"receipt_id":"draft","receipt_version":2,"amount_cents":100,"summary_version":0}),
        "p",
        100,
    );
    assert!(result.unwrap_err().1.contains("草稿"));
}
#[test]
fn stale_or_unconfirmed_updates_preserve_receipt_and_application_snapshot() {
    let mut s = setup();
    save(&mut s, "wechat", "first", "").unwrap();
    let first = s.users["alice"].receipts[0].clone();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../api-contract/fixtures/receipt-confirmed.json"
    ))
    .unwrap();
    assert_eq!(first, fixture);
    s.withdrawals.applications.insert(
        "a".into(),
        json!({"owner":"alice","receipt_snapshot":first}),
    );
    let v = s.users["alice"].version;
    assert!(
        s.command(
            "alice",
            "receipt",
            &json!({"method":"wechat","file_id":"next","confirmed":false,"version":v}),
            "fail",
            "r",
            "now"
        )
        .is_err()
    );
    assert!(
        s.command(
            "alice",
            "receipt",
            &json!({"method":"wechat","file_id":"next","confirmed":true,"version":v-1}),
            "fail",
            "r",
            "now"
        )
        .is_err()
    );
    save(&mut s, "wechat", "second", "").unwrap();
    assert_eq!(s.withdrawals.applications["a"]["receipt_snapshot"], first);
    let view = s.read("alice", "receipts", "alice", "r", "now").unwrap();
    assert_eq!(view[1]["is_default"], true);
    assert!(s.read("bob", "receipts", "alice", "r", "now").is_err());
}
