use crowd_api::{identity::domain::User, platform::domain::State, promotion::domain::Code};
use serde_json::{Value, json};

fn state() -> State {
    // Existing persisted keys remain readable without a destructive migration.
    serde_json::from_value(json!({
        "users": {"alice": {
            "id":"alice", "nickname":"Alice", "avatar":null, "version":1,
            "status":"active", "roles":["user"], "scope":["alice"],
            "receipts":[], "restrictions":[], "agreement":{"version":"v1"}, "source":null
        }},
        "rules":[], "codes":{"original":{
            "owner":"alice", "version":1, "active":true, "product":null, "task":null
        }},
        "cases":[], "audit":[], "idempotency":{}
    }))
    .unwrap()
}

fn command(state: &mut State, action: &str, input: Value, id: &str) -> Value {
    state
        .command(
            "alice",
            action,
            &input,
            id,
            "request",
            "2026-10-28T00:00:00Z",
        )
        .unwrap()
}

#[test]
fn historical_document_and_legacy_namespace_keep_identical_serialization() {
    let original = state();
    let _: &User = &original.users["alice"];
    let _: &Code = &original.codes["original"];
    let stored = serde_json::to_value(&original).unwrap();
    let legacy: crowd_api::community::domain::State =
        serde_json::from_value(stored.clone()).unwrap();
    assert_eq!(serde_json::to_value(legacy).unwrap(), stored);
    assert!(original.tasks.instances.is_empty());
    assert!(original.catalog.products.is_empty());
}

#[test]
fn domain_dispatch_preserves_noop_audit_and_append_only_receipt_history() {
    let mut state = state();
    let code = command(&mut state, "code_create", json!({}), "unused");
    assert_eq!(code["id"], "original");
    assert!(state.audit.is_empty());
    command(
        &mut state,
        "receipt",
        json!({
            "method":"wechat", "file_id":"private-file", "confirmed":true, "version":1
        }),
        "receipt",
    );
    let snapshot = state.users["alice"].receipts[0].clone();
    command(
        &mut state,
        "receipt_disable",
        json!({"id":"receipt", "version":2}),
        "disable",
    );
    let count = state.audit.len();
    let version = state.users["alice"].version;
    command(
        &mut state,
        "receipt_disable",
        json!({"id":"receipt", "version":0}),
        "duplicate",
    );
    assert_eq!(state.audit.len(), count);
    assert_eq!(state.users["alice"].version, version);
    assert_eq!(state.users["alice"].receipts[0], snapshot);
    assert_eq!(state.users["alice"].receipts.len(), 2);
    assert_eq!(state.codes["original"].owner, "alice");
}

#[test]
fn moved_rule_and_audit_owners_keep_permission_and_data_scope_checks() {
    let mut state = state();
    let input =
        json!({"kind":"agreements", "content":{"service":"v1", "privacy":"v1"}, "version":0});
    assert_eq!(
        state
            .check_command_access("alice", "rule_save", &input)
            .unwrap_err()
            .0,
        "FORBIDDEN"
    );
    assert!(state.rules.is_empty());
    assert_eq!(
        state.read("alice", "audit", "*", "r", "now").unwrap_err().0,
        "FORBIDDEN"
    );
    state
        .users
        .get_mut("alice")
        .unwrap()
        .roles
        .push("auditor".into());
    state.log("alice", "own", "alice", "r", "now");
    state.log("other", "other", "other", "r", "now");
    let records = state.read("alice", "audit", "alice", "r", "now").unwrap();
    assert_eq!(records.as_array().unwrap().len(), 1);
    assert_eq!(records[0]["object"], "alice");
    assert_eq!(
        state
            .read("alice", "audit", "other", "r", "now")
            .unwrap_err()
            .0,
        "FORBIDDEN"
    );
}
