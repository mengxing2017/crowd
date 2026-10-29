use crate::shared::{CommandContext, Result};
use crate::support::south::port::DomainPort;
use serde_json::Value;
pub(crate) fn apply<P: DomainPort + ?Sized>(s: &mut P, c: &CommandContext<'_>) -> Result<Value> {
    let result = crate::support::domain::tickets::command(
        s,
        c.actor,
        c.action,
        c.input,
        c.generated,
        c.epoch,
    )?;
    s.log(
        c.actor,
        c.action,
        c.input["id"].as_str().unwrap_or(c.actor),
        c.request,
        c.now,
    );
    s.annotate_audit(serde_json::json!({"requirement":"M23","result_ref":result["id"],"version":result["version"]}));
    Ok(result)
}
