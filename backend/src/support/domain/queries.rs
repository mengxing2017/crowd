use crate::shared::{Failure, Result};
use crate::support::south::port::DomainPort;
use serde_json::{Value, json};
pub(crate) fn read<P: DomainPort + ?Sized>(
    state: &P,
    actor: &str,
    section: &str,
    _target: &str,
) -> Result<Value> {
    match section {
        "cases" => Ok(json!(
            state
                .cases()
                .iter()
                .filter(|v| v["owner"] == actor)
                .collect::<Vec<_>>()
        )),
        _ => Err(Failure("NOT_FOUND", "接口不存在")),
    }
}
