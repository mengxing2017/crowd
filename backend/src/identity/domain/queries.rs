use super::BehaviorRules;
use crate::identity::south::port::DomainPort;
use crate::shared::mask_receipts;
use crate::shared::{Failure, Result};
use serde_json::{Value, json};
pub(crate) fn read<P: DomainPort + ?Sized>(
    state: &P,
    actor: &str,
    section: &str,
    target: &str,
) -> Result<Value> {
    match section {
        "me" => {
            let mut value = serde_json::to_value(state.user(actor)?).expect("serialize user");
            value["receipts"] = json!(mask_receipts(&state.user(actor)?.receipts));
            Ok(value)
        }
        "users" => {
            state.authorize(actor, "identity_reviewer", target)?;
            let u = state.user(target)?;
            Ok(
                json!({"id":u.id,"status":u.status,"version":u.version,"roles":u.roles,"scope":u.scope,"restrictions":u.restrictions}),
            )
        }
        _ => Err(Failure("NOT_FOUND", "接口不存在")),
    }
}
