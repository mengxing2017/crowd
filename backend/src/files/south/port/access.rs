use serde_json::Value;
use std::collections::BTreeMap;
pub trait FileAccess {
    fn task_attachment_exists(&self, id: &str) -> bool;
    fn settlement_applications(&self) -> &BTreeMap<String, Value>;
}
