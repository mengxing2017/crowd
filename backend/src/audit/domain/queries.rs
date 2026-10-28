use crate::audit::south::port::DomainPort;
use crate::shared::{Failure, Result};
use serde_json::{Value, json};
pub(crate) fn read<P: DomainPort + ?Sized>(
    state: &P,
    actor: &str,
    section: &str,
    target: &str,
) -> Result<Value> {
    match section {
        "audit" => {
            state.authorize(actor, "auditor", target)?;
            Ok(json!(
                state
                    .audit()
                    .iter()
                    .filter(|a| target == "*" || a["object"] == target)
                    .rev()
                    .take(100)
                    .collect::<Vec<_>>()
            ))
        }
        _ => Err(Failure("NOT_FOUND", "接口不存在")),
    }
}
