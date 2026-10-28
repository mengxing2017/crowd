use crate::platform::domain::State;
use crate::shared::{Failure, Result, text};
use serde_json::Value;
pub(crate) fn reveal(
    state: &mut State,
    actor: &str,
    input: &Value,
    request: &str,
    now: &str,
) -> Result<Value> {
    text(input, "reason")?;
    let id = text(input, "id")?;
    let receipt = state
        .user(actor)?
        .receipts
        .iter()
        .find(|v| v["id"] == id)
        .cloned()
        .ok_or(Failure("NOT_FOUND", "资料不存在"))?;
    state.log(actor, "REQ-03-008", id, request, now);
    Ok(receipt)
}
