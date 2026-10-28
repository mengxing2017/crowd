use crate::commissions::south::port::CommandContext;
use crate::commissions::south::port::DomainPort;
use crate::shared::Result;
use serde_json::Value;
pub(crate) fn apply<P: DomainPort + ?Sized>(
    state: &mut P,
    c: &CommandContext<'_>,
) -> Result<Value> {
    let previous_entries = state.commissions().entries.len();
    let result = crate::commissions::domain::command(
        state,
        c.actor,
        c.action,
        c.input,
        c.generated,
        c.epoch,
    )?;
    for entry in state
        .commissions_mut()
        .entries
        .iter_mut()
        .skip(previous_entries)
    {
        entry["request_id"] = serde_json::json!(c.request);
    }
    state.log(
        c.actor,
        c.action,
        c.input["id"].as_str().unwrap_or(c.actor),
        c.request,
        c.now,
    );
    {
        let mut log = serde_json::json!({});
        log["reason"] = c.input["reason"].clone();
        log["implementation_version"] = serde_json::json!("M13-M16-v1");
        log["rule_version"] = result["rule"]["version"].clone();
        log["result_ref"] = result["id"].clone();
        log["object_version"] = result["version"].clone();

        state.annotate_audit(log);
    }
    Ok(result)
}
