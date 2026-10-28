use crate::promotion::south::port::DomainPort;
use crate::shared::{Failure, Result, text};
use serde_json::Value;
pub(crate) fn access<P: DomainPort + ?Sized>(
    state: &P,
    actor: &str,
    action: &str,
    input: &Value,
) -> Result<()> {
    if action == "code_rotate" {
        let code = state
            .codes()
            .get(text(input, "id")?)
            .ok_or(Failure("NOT_FOUND", "推广码不存在"))?;
        state.authorize(actor, "promotion_manager", &code.owner)?;
    }
    Ok(())
}
