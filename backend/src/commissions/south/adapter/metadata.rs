use crate::commissions::south::port::LedgerMetadata;
use crate::platform::south::port::State;
impl LedgerMetadata for State {
    fn commission_checkpoint(&self) -> usize {
        self.commissions.entries.len()
    }
    fn stamp_commission_request(&mut self, checkpoint: usize, request: &str) {
        for entry in self.commissions.entries.iter_mut().skip(checkpoint) {
            entry["request_id"] = serde_json::json!(request);
        }
    }
}
