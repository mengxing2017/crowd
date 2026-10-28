use crate::files::south::port::FileAccess;
use crate::platform::south::port::State;
use serde_json::Value;
use std::collections::BTreeMap;
impl FileAccess for State {
    fn task_attachment_exists(&self, id: &str) -> bool {
        self.tasks.attachments.contains_key(id)
    }
    fn settlement_applications(&self) -> &BTreeMap<String, Value> {
        &self.withdrawals.applications
    }
}
