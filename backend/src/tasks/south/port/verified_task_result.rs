use crate::platform::domain::Result;
use serde_json::Value;
/// Trusted source adapter boundary. Never exposed as a user HTTP command.
/// Adapter must authenticate/signature-check the source before invoking this port.
pub trait VerifiedTaskResult {
    fn record_task_result(
        &mut self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
        base_cents: Option<i64>,
    ) -> Result<Value>;
}
