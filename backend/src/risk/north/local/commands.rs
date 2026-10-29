use crate::risk::south::port::DomainPort;
use crate::shared::{CommandContext, Result};
use serde_json::{Value, json};
pub(crate) fn apply<P: DomainPort + ?Sized>(s: &mut P, c: &CommandContext<'_>) -> Result<Value> {
    let result = crate::risk::domain::command(s, c.actor, c.action, c.input, c.generated, c.epoch)?;
    s.log(
        c.actor,
        c.action,
        c.input["id"].as_str().unwrap_or(c.actor),
        c.request,
        c.now,
    );
    s.annotate_audit(json!({"reason":c.input["reason"],"implementation_version":"M20-M24-v1","result_ref":result["id"],"object_version":result["version"]}));
    Ok(result)
}
