use crate::platform::{
    domain::{Result, State},
    north::local::CommandContext,
};
use serde_json::Value;
pub(crate) fn apply(state: &mut State, c: &CommandContext<'_>) -> Result<Value> {
    let previous_entries = state.commissions.entries.len();
    let result = crate::settlements::domain::command(
        state,
        c.actor,
        c.action,
        c.input,
        c.generated,
        c.epoch,
    )?;
    for entry in state.commissions.entries.iter_mut().skip(previous_entries) {
        entry["request_id"] = serde_json::json!(c.request);
    }
    state.log(
        c.actor,
        c.action,
        c.input["id"].as_str().unwrap_or(c.actor),
        c.request,
        c.now,
    );
    if let Some(log) = state.audit.last_mut() {
        log["reason"] = c.input["reason"].clone();
        log["implementation_version"] = serde_json::json!("M13-M16-v1");
        log["rule_version"] = result["rule"]["version"].clone();
        log["result_ref"] = result["id"].clone();
        log["object_version"] = result["version"].clone();
    }
    Ok(result)
}
