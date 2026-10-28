use crate::platform::domain::State;
use crate::shared::{Failure, Result, text};
use serde_json::Value;
pub(crate) fn access(state: &State, actor: &str, action: &str, input: &Value) -> Result<()> {
    if action == "code_rotate" {
        let code = state
            .codes
            .get(text(input, "id")?)
            .ok_or(Failure("NOT_FOUND", "推广码不存在"))?;
        state.authorize(actor, "promotion_manager", &code.owner)?;
    }
    Ok(())
}
