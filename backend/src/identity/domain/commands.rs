use super::BehaviorRules;
use crate::identity::south::port::DomainPort;
use crate::shared::{Failure, Result, check_version, invalid, text};
use serde_json::{Value, json};
pub(crate) fn command<P: DomainPort + ?Sized>(
    state: &mut P,
    actor: &str,
    action: &str,
    input: &Value,
    _generated: &str,
    request: &str,
    time: &str,
) -> Result<Value> {
    let result = match action {
        "profile" => {
            let nickname = text(input, "nickname")?;
            let u = state.users_mut().get_mut(actor).unwrap();
            check_version(u.version, input)?;
            u.nickname = nickname.into();
            u.avatar = input["avatar"].as_str().map(str::to_owned);
            u.version += 1;
            json!({"version":u.version})
        }
        "access" | "restriction" => {
            let target = text(input, "user_id")?;
            state.authorize(actor, "identity_reviewer", target)?;
            if target == actor {
                return Err(Failure("FORBIDDEN", "不能修改自己的权限或限制"));
            }
            text(input, "reason")?;
            let u = state
                .users_mut()
                .get_mut(target)
                .ok_or(Failure("NOT_FOUND", "用户不存在"))?;
            check_version(u.version, input)?;
            if action == "access" {
                let roles: Vec<String> = serde_json::from_value(input["roles"].clone())
                    .map_err(|_| invalid("角色格式错误"))?;
                if roles.iter().any(|r| {
                    ![
                        "user",
                        "rule_manager",
                        "identity_reviewer",
                        "promotion_manager",
                        "auditor",
                        "operator",
                        "product_reviewer",
                        "product_emergency",
                        "attribution_reviewer",
                        "task_reviewer",
                        "settlement_reviewer",
                        "manual_handler",
                        "verifier",
                        "support",
                    ]
                    .contains(&r.as_str())
                }) {
                    return Err(invalid("未知角色"));
                }
                u.roles = roles;
                u.scope = serde_json::from_value(input["scope"].clone())
                    .map_err(|_| invalid("数据范围格式错误"))?;
            } else {
                let scope = text(input, "restriction_scope")?;
                if !["all", "withdrawal", "promotion"].contains(&scope) {
                    return Err(invalid("限制范围无效"));
                }
                u.restrictions.push(
                    json!({"scope":scope,"reason":input["reason"],"appeal":"联系客服","time":time}),
                );
                if scope == "all" {
                    u.status = "disabled".into();
                }
            }
            u.version += 1;
            json!({"version":u.version})
        }
        "phone" => {
            return Err(Failure(
                "FEATURE_NOT_ENABLED",
                "手机号绑定策略及可信验证渠道尚未启用，原绑定保持不变",
            ));
        }
        _ => return Err(Failure("NOT_FOUND", "操作不存在")),
    };
    state.log(actor, action, actor, request, time);
    Ok(result)
}
