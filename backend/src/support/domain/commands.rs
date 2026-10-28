use crate::platform::domain::State;
use crate::shared::{Failure, Result, text};
use serde_json::{Value, json};
pub(crate) fn command(
    state: &mut State,
    actor: &str,
    action: &str,
    input: &Value,
    generated: &str,
    request: &str,
    time: &str,
) -> Result<Value> {
    let result = match action {
        "correction" | "closure" | "merge" => {
            text(input, "reason")?;
            // No account deletion or identity merge before dependent ledgers can be checked.
            let status = if action == "closure" || action == "merge" {
                "pending_dependency_check"
            } else {
                "pending_verification"
            };
            state.cases.push(json!({"id":generated,"owner":actor,"kind":action,"status":status,"reason":input["reason"],"affected_application":input["application_id"],"time":time}));
            json!({"id":generated,"status":status})
        }
        _ => return Err(Failure("NOT_FOUND", "操作不存在")),
    };
    state.log(actor, action, actor, request, time);
    Ok(result)
}
