use crate::platform::domain::State;
use crate::shared::{Result, text};
use serde_json::Value;
pub(crate) fn validate_input(action: &str, input: &Value) -> Result<()> {
    if action == "profile" {
        crate::shared::validate_fields(input, &["nickname", "avatar", "version"])?;
    }
    Ok(())
}
pub(crate) fn access(state: &State, actor: &str, action: &str, input: &Value) -> Result<()> {
    match action {
        "access" | "restriction" => {
            state.authorize(actor, "identity_reviewer", text(input, "user_id")?)
        }
        _ => Ok(()),
    }
}
