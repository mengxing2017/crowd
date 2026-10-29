use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Risk {
    pub file_fingerprints: BTreeMap<String, String>,
    pub records: BTreeMap<String, Value>,
    pub history: Vec<Value>,
}
