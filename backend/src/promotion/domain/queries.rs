use crate::platform::domain::State;
use crate::shared::{Failure, Result};
use serde_json::{Value, json};
pub(crate) fn read(state: &State, actor: &str, section: &str, target: &str) -> Result<Value> {
    match section {
        "codes" => Ok(json!(
            state
                .codes
                .iter()
                .filter(|(_, c)| c.owner == actor)
                .map(|(id, c)| json!({"id":id,"code":c}))
                .collect::<Vec<_>>()
        )),
        "managed_codes" => {
            state.authorize(actor, "promotion_manager", target)?;
            Ok(json!(
                state
                    .codes
                    .iter()
                    .filter(|(_, c)| c.owner == target)
                    .map(|(id, c)| json!({"id":id,"code":c}))
                    .collect::<Vec<_>>()
            ))
        }
        _ => Err(Failure("NOT_FOUND", "接口不存在")),
    }
}
