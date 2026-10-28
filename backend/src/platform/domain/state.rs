use crate::identity::domain::User;
use crate::promotion::domain::Code;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct State {
    #[serde(default)]
    pub commissions: crate::commissions::domain::Commissions,
    #[serde(default)]
    pub withdrawals: crate::withdrawals::domain::Withdrawals,
    #[serde(default)]
    pub tasks: crate::tasks::domain::Tasks,
    #[serde(default)]
    pub catalog: crate::catalog::domain::Catalog,
    pub users: BTreeMap<String, User>,
    pub rules: Vec<Value>,
    pub codes: BTreeMap<String, Code>,
    pub cases: Vec<Value>,
    pub audit: Vec<Value>,
    pub idempotency: BTreeMap<String, Value>,
}
