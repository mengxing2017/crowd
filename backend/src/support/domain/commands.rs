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
            if let Some(record) = input["record_id"].as_str().filter(|v| !v.is_empty())
                && !state
                    .commissions
                    .entries
                    .iter()
                    .any(|e| e["id"] == record && e["owner"] == actor)
            {
                return Err(Failure("FORBIDDEN", "数字记录不存在或不属于本人"));
            }
            if let Some(id) = input["application_id"].as_str().filter(|v| !v.is_empty())
                && state.withdrawals.applications.contains_key(id)
                && state.withdrawals.applications[id]["owner"] != actor
            {
                return Err(Failure("FORBIDDEN", "申请不属于本人"));
            }
            // No account deletion or identity merge before dependent ledgers can be checked.
            let status = if action == "closure" || action == "merge" {
                "pending_dependency_check"
            } else {
                "pending_verification"
            };
            state.cases.push(json!({"id":generated,"owner":actor,"kind":action,"status":status,"reason":input["reason"],"affected_application":input["application_id"],"record_id":input["record_id"],"verification_kind":if input["record_id"].is_string(){"digital_correction"}else{"offline_payment_inquiry"},"time":time}));
            json!({"id":generated,"status":status})
        }
        _ => return Err(Failure("NOT_FOUND", "操作不存在")),
    };
    state.log(actor, action, actor, request, time);
    Ok(result)
}
