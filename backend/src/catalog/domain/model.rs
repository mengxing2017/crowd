use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Catalog {
    pub categories: BTreeMap<String, Value>,
    pub products: BTreeMap<String, Value>,
    pub policies: Vec<Value>,
    pub touches: BTreeMap<String, Value>,
    pub claims: BTreeMap<String, Value>,
    pub disputes: BTreeMap<String, Value>,
    pub completions: BTreeMap<String, Value>,
    pub notifications: Vec<Value>,
    pub eligibility: BTreeMap<String, Value>,
    pub revision: u64,
}
