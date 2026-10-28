use super::validation::validate_catalogue;
use crate::governance::south::port::DomainPort;
use crate::shared::{Failure, Result, check_version, invalid, text};
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
        "rule_save" => {
            state.authorize(actor, "rule_manager", "platform")?;
            let kind = text(input, "kind")?;
            if ![
                "business_chain",
                "operator",
                "roles",
                "completion",
                "review",
                "product_tasks",
                "attribution",
                "digital_boundary",
                "amount_unit",
                "release_scope",
                "agreements",
            ]
            .contains(&kind)
            {
                return Err(invalid("规则类型无效"));
            }
            if !input["content"].is_object() {
                return Err(invalid("规则内容必须为对象"));
            }
            let version = state.rules().iter().filter(|r| r["kind"] == kind).count() as u64;
            check_version(version, input)?;
            state.rules_mut().push(json!({"id":generated,"kind":kind,"version":version+1,"status":"draft","content":input["content"],"owner":actor,"time":time}));
            json!({"id":generated,"version":version+1})
        }
        "rule_publish" | "rule_disable" => {
            state.authorize(actor, "rule_manager", "platform")?;
            let id = text(input, "id")?;
            let pos = state
                .rules()
                .iter()
                .position(|r| r["id"] == id)
                .ok_or(Failure("NOT_FOUND", "规则版本不存在"))?;
            let r = state.rules()[pos].clone();
            check_version(r["version"].as_u64().unwrap(), input)?;
            let expected = if action == "rule_publish" {
                "draft"
            } else {
                "published"
            };
            if r["status"] != expected {
                return Err(Failure("VERSION_CONFLICT", "规则状态已改变"));
            }
            if action == "rule_publish" {
                if r["kind"] == "digital_boundary" && r["content"]["manual_only"] != true {
                    return Err(invalid("仅支持数字记录与人工线下结算"));
                }
                if r["kind"] == "product_tasks" {
                    validate_catalogue(&r["content"])?;
                }
                if r["kind"] == "agreements" {
                    text(&r["content"], "service")?;
                    text(&r["content"], "privacy")?;
                }
                for other in state.rules_mut() {
                    if other["kind"] == r["kind"] && other["status"] == "published" {
                        other["status"] = json!("disabled");
                    }
                }
            }
            state.rules_mut()[pos]["status"] = json!(if action == "rule_publish" {
                "published"
            } else {
                "disabled"
            });
            json!({"id":id,"status":state.rules()[pos]["status"]})
        }
        _ => return Err(Failure("NOT_FOUND", "操作不存在")),
    };
    state.log(actor, action, actor, request, time);
    Ok(result)
}
