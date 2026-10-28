use super::BehaviorRules;
use super::Code;
use crate::promotion::south::port::DomainPort;
use crate::shared::{Failure, Result, check_version, text};
use serde_json::{Value, json};
pub(crate) fn command<P: DomainPort + ?Sized>(
    state: &mut P,
    actor: &str,
    action: &str,
    input: &Value,
    generated: &str,
    request: &str,
    time: &str,
) -> Result<Value> {
    let result = match action {
        "code_create" => {
            let product = input["product"].as_str();
            let task = input["task"].as_str();
            state.validate_target(product, task)?;
            if state
                .user(actor)?
                .restrictions
                .iter()
                .any(|r| r["scope"] == "promotion")
            {
                return Err(Failure(
                    "ACCOUNT_RESTRICTED",
                    "推广功能受限，请联系客服申诉",
                ));
            }
            if let Some((id, _)) = state.codes().iter().find(|(_, c)| {
                c.owner == actor
                    && c.active
                    && c.product.as_deref() == product
                    && c.task.as_deref() == task
            }) {
                return Ok(json!({"id":id}));
            }
            if state.codes().contains_key(generated) {
                return Err(Failure("CODE_COLLISION", "推广码冲突，请重试"));
            }
            state.codes_mut().insert(
                generated.into(),
                Code {
                    owner: actor.into(),
                    version: 1,
                    active: true,
                    product: product.map(str::to_owned),
                    task: task.map(str::to_owned),
                },
            );
            json!({"id":generated})
        }
        "code_rotate" => {
            let id = text(input, "id")?;
            let reason = text(input, "reason")?;
            let approval = text(input, "approval")?;
            let c = state
                .codes()
                .get(id)
                .ok_or(Failure("NOT_FOUND", "推广码不存在"))?
                .clone();
            state.authorize(actor, "promotion_manager", &c.owner)?;
            if !c.active {
                return Err(Failure("VERSION_CONFLICT", "原推广码已经撤销"));
            }
            check_version(c.version, input)?;
            if state.codes().contains_key(generated) {
                return Err(Failure("CODE_COLLISION", "推广码冲突，请重试"));
            }
            state.codes_mut().get_mut(id).unwrap().active = false;
            state.codes_mut().insert(
                generated.into(),
                Code {
                    version: c.version + 1,
                    ..c
                },
            );
            state.log(actor, "REQ-04-008", id, request, time);
            state.annotate_audit(serde_json::json!({"reason":json!(reason)}));
            state.annotate_audit(serde_json::json!({"approval":json!(approval)}));
            state.annotate_audit(serde_json::json!({"approved_by":json!(actor)}));
            json!({"id":generated})
        }
        _ => return Err(Failure("NOT_FOUND", "操作不存在")),
    };
    state.log(actor, action, actor, request, time);
    Ok(result)
}
