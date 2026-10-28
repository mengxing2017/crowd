use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Withdrawals {
    pub rules: Vec<Value>,
    pub applications: BTreeMap<String, Value>,
    pub previews: BTreeMap<String, Value>,
}
