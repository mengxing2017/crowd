//! Completion adapters must authenticate their source before invoking this port.
//! No HTTP completion endpoint is exposed until a verified source adapter is configured.
use crate::platform::south::port::rules::Result;
use serde_json::Value;
pub trait CompletionRecorder {
    fn record_completion(
        &mut self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
    ) -> Result<Value>;
}
