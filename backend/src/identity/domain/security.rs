use crate::identity::south::port::DomainPort;
use crate::shared::{Failure, Result, number, require, text, version};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Security {
    pub policy: Value,
    pub grants: BTreeMap<String, Value>,
    pub history: Vec<Value>,
}
pub(crate) fn is_security_command(a: &str) -> bool {
    matches!(
        a,
        "security_policy_publish"
            | "temporary_grant_request"
            | "temporary_grant_review"
            | "temporary_grant_revoke"
    )
}
pub(crate) fn security_access<P: DomainPort + ?Sized>(
    s: &P,
    a: &str,
    action: &str,
    i: &Value,
) -> Result<()> {
    use super::BehaviorRules;
    let target = if action == "security_policy_publish" {
        "security"
    } else if action == "temporary_grant_request" {
        text(i, "user_id")?
    } else {
        s.security()
            .grants
            .get(text(i, "id")?)
            .and_then(|v| v["user_id"].as_str())
            .ok_or(Failure("NOT_FOUND", "临时授权不存在"))?
    };
    s.authorize(a, "identity_reviewer", target)
}
pub(crate) fn security_command<P: DomainPort + ?Sized>(
    s: &mut P,
    a: &str,
    action: &str,
    i: &Value,
    id: &str,
    now: i64,
) -> Result<Value> {
    security_access(s, a, action, i)?;
    text(i, "reason")?;
    let v = if action == "security_policy_publish" {
        let old = if s.security().policy.is_null() {
            json!({"version":0})
        } else {
            s.security().policy.clone()
        };
        version(&old, i)?;
        require(number(i, "reauth_seconds")? > 0, "须明确再次验证有效期")?;
        let actions = i["actions"]
            .as_array()
            .ok_or(Failure("INVALID_INPUT", "须明确高风险动作"))?;
        require(
            !actions.is_empty()
                && actions.iter().all(|v| {
                    matches!(
                        v.as_str(),
                        Some(
                            "access"
                                | "restriction"
                                | "security_policy_publish"
                                | "temporary_grant_review"
                                | "settlement_register"
                                | "settlement_verify"
                                | "adjustment_batch_review"
                                | "privacy_close_review"
                                | "privacy_repair_review"
                                | "export_request"
                                | "product_offline"
                        )
                    )
                }),
            "包含不支持的高风险动作",
        )?;
        json!({"id":"security","version":old["version"].as_u64().unwrap_or(0)+1,"reauth_seconds":i["reauth_seconds"],"actions":actions,"publisher":a,"published_at":now})
    } else if action == "temporary_grant_request" {
        let target = text(i, "user_id")?;
        require(target != a, "不能申请自己的应急权限")?;
        require(
            s.users().get(target).is_some_and(|u| u.status == "active"),
            "授权对象无效",
        )?;
        let roles: Vec<String> = serde_json::from_value(i["roles"].clone())
            .map_err(|_| Failure("INVALID_INPUT", "角色无效"))?;
        require(
            !roles.is_empty() && roles.iter().all(|r| super::commands::valid_role(r)),
            "临时角色无效",
        )?;
        let scope: Vec<String> = serde_json::from_value(i["scope"].clone())
            .map_err(|_| Failure("INVALID_INPUT", "范围无效"))?;
        require(
            !scope.is_empty() && scope.iter().all(|s| !s.trim().is_empty()),
            "必须明确临时授权范围",
        )?;
        require(number(i, "expires_at")? > now, "授权期限必须在未来")?;
        // An approver may delegate only capabilities they currently possess for every target.
        for role in &roles {
            for target in &scope {
                super::BehaviorRules::authorize(s, a, role, target)?;
            }
        }
        json!({"id":id,"version":1,"status":"pending","user_id":target,"roles":roles,"scope":scope,"expires_at":i["expires_at"],"requester":a,"reason":i["reason"],"created_at":now})
    } else {
        let mut v = s
            .security()
            .grants
            .get(text(i, "id")?)
            .cloned()
            .ok_or(Failure("NOT_FOUND", "授权不存在"))?;
        version(&v, i)?;
        if action == "temporary_grant_review" {
            require(
                v["status"] == "pending" && v["requester"] != a && v["user_id"] != a,
                "需独立审批，不能自批临时权限",
            )?;
            require(number(&v, "expires_at")? > now, "授权期限已过")?;
            if i["approve"] == true {
                for r in v["roles"].as_array().unwrap() {
                    for t in v["scope"].as_array().unwrap() {
                        super::BehaviorRules::authorize(
                            s,
                            a,
                            r.as_str().unwrap_or(""),
                            t.as_str().unwrap_or(""),
                        )?;
                    }
                }
            }
            v["status"] = json!(if i["approve"] == true {
                "approved"
            } else {
                "rejected"
            });
            v["approver"] = json!(a);
        } else {
            require(v["status"] == "approved", "授权不在生效状态")?;
            v["status"] = json!("revoked");
        }
        v["version"] = json!(number(&v, "version")? + 1);
        v["updated_at"] = json!(now);
        v
    };
    s.security_mut()
        .history
        .push(json!({"action":action,"actor":a,"at":now,"record":v}));
    if action == "security_policy_publish" {
        s.security_mut().policy = v.clone();
    } else {
        s.security_mut()
            .grants
            .insert(text(&v, "id")?.into(), v.clone());
    }
    Ok(v)
}
pub(crate) fn security_read<P: DomainPort + ?Sized>(s: &P, a: &str) -> Result<Value> {
    use super::BehaviorRules;
    s.authorize(a, "identity_reviewer", "security")?;
    Ok(
        json!({"policy":s.security().policy,"grants":s.security().grants.values().filter(|v|s.authorize(a,"identity_reviewer",v["user_id"].as_str().unwrap_or("")).is_ok()).collect::<Vec<_>>()}),
    )
}
