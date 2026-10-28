use super::BehaviorRules;
use crate::identity::south::port::DomainPort;
use crate::shared::{Result, text};
use serde_json::Value;
pub(crate) fn validate_input(action: &str, input: &Value) -> Result<()> {
    if action == "profile" {
        crate::shared::validate_fields(input, &["nickname", "avatar", "version"])?;
    }
    Ok(())
}
pub(crate) fn access<P: DomainPort + ?Sized>(
    state: &P,
    actor: &str,
    action: &str,
    input: &Value,
) -> Result<()> {
    match action {
        "access" | "restriction" => {
            state.authorize(actor, "identity_reviewer", text(input, "user_id")?)
        }
        _ => Ok(()),
    }
}
