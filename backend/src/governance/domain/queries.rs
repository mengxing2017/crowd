use crate::governance::south::port::DomainPort;
use crate::shared::{Failure, Result};
use serde_json::{Value, json};
pub(crate) fn read<P: DomainPort + ?Sized>(
    state: &P,
    actor: &str,
    section: &str,
    _target: &str,
) -> Result<Value> {
    match section {
        "rules" => {
            state.authorize(actor, "rule_manager", "platform")?;
            Ok(json!(state.rules()))
        }
        _ => Err(Failure("NOT_FOUND", "接口不存在")),
    }
}
