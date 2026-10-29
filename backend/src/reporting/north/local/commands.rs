use crate::reporting::south::port::DomainPort;
use crate::shared::{CommandContext, Result};
use serde_json::{Value, json};
pub(crate) fn apply<P: DomainPort + ?Sized>(s: &mut P, c: &CommandContext<'_>) -> Result<Value> {
    let result =
        crate::reporting::domain::command(s, c.actor, c.action, c.input, c.generated, c.epoch)?;
    s.log(
        c.actor,
        c.action,
        c.input["id"].as_str().unwrap_or(c.actor),
        c.request,
        c.now,
    );
    s.annotate_audit(json!({"implementation_version":"M25-M29-v1","object_version":result["version"],"result_ref":result["id"]}));
    Ok(result)
}
