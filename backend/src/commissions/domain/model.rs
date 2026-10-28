use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Commissions {
    pub entries: Vec<Value>,
    pub proposals: BTreeMap<String, Value>,
    pub disputes: BTreeMap<String, Value>,
}
