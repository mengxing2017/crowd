use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Tasks {
    pub reminders: Vec<String>,
    pub attachments: BTreeMap<String, Value>,
    pub verified_results: BTreeMap<String, Value>,
    pub prices: BTreeMap<String, Value>,
    pub templates: BTreeMap<String, Value>,
    pub instances: BTreeMap<String, Value>,
    pub orders: BTreeMap<String, String>,
    pub confirmations: BTreeMap<String, Value>,
    pub changes: Vec<Value>,
    pub inspections: Vec<Value>,
    pub proposals: BTreeMap<String, Value>,
}
