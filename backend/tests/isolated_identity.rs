use crowd_api::{
    audit::south::port::AuditWriter,
    identity::{
        south::port::DomainPort,
        south::port::rules::{BehaviorRules, User},
    },
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

// A domain can run without the platform transaction envelope, PostgreSQL or
// any other domain's entities. Unexpected integration writes fail immediately.
#[derive(Default)]
struct IdentityMemory {
    users: BTreeMap<String, User>,
    security: crate::identity::south::port::rules::Security,
}
impl DomainPort for IdentityMemory {
    fn security(&self) -> &crate::identity::south::port::rules::Security {
        &self.security
    }
    fn security_mut(&mut self) -> &mut crate::identity::south::port::rules::Security {
        &mut self.security
    }
    fn epoch(&self) -> i64 {
        100
    }
    fn users(&self) -> &BTreeMap<String, User> {
        &self.users
    }
    fn users_mut(&mut self) -> &mut BTreeMap<String, User> {
        &mut self.users
    }
}
impl AuditWriter for IdentityMemory {
    fn log(&mut self, _: &str, _: &str, _: &str, _: &str, _: &str) {
        panic!("unexpected audit write")
    }
    fn log_failure(&mut self, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str) {
        panic!("unexpected audit write")
    }
    fn annotate_audit(&mut self, _: Value) {
        panic!("unexpected audit write")
    }
}
#[test]
fn identity_rules_run_against_their_own_port_without_global_state() {
    let mut identity = IdentityMemory::default();
    identity.register_user("alice", json!({"version":1}), None);
    identity.authorize("alice", "user", "alice").unwrap();
    assert_eq!(
        identity.authorize("alice", "user", "bob").unwrap_err().0,
        "FORBIDDEN"
    );
    assert_eq!(
        identity
            .authorize("alice", "operator", "alice")
            .unwrap_err()
            .0,
        "FORBIDDEN"
    );
    identity.users.get_mut("alice").unwrap().nickname = "已有资料".into();
    identity.register_user("alice", json!({"version":2}), Some("another-source".into()));
    assert_eq!(identity.users.len(), 1);
    assert_eq!(identity.user("alice").unwrap().nickname, "已有资料");
    assert_eq!(
        identity.user("alice").unwrap().agreement,
        json!({"version":1})
    );
    identity.users.get_mut("alice").unwrap().status = "disabled".into();
    assert_eq!(
        identity.authorize("alice", "user", "alice").unwrap_err().0,
        "ACCOUNT_RESTRICTED"
    );
    identity.users.get_mut("alice").unwrap().status = "closed".into();
    assert_eq!(identity.user("alice").err().unwrap().0, "SESSION_EXPIRED");
}
