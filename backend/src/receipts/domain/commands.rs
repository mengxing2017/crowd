use crate::platform::domain::State;
use crate::shared::{Failure, Result, check_version, invalid, text};
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
        "receipt" => {
            let method = text(input, "method")?;
            if !["wechat", "alipay"].contains(&method) {
                return Err(invalid("收款方式无效"));
            }
            if input["confirmed"] != true {
                return Err(invalid("请主动确认资料可供本人收款"));
            }
            let mut record = json!({"id":generated,"method":method,"status":"confirmed","time":time,"owner":actor,"official_verified":false,"declaration_version":"receipt-self-v1","confirmed_at":time});
            if method == "wechat" {
                record["file_id"] = json!(text(input, "file_id")?);
            } else {
                // Without an explicit published policy, retain a draft only.
                record["account"] = json!(text(input, "account")?);
                if !input["name"].is_null() {
                    text(input, "name")?;
                    record["name"] = input["name"].clone();
                }
                record["status"] = json!("draft");
                if let Some(rule) = super::policy(state)
                    && rule["methods"]
                        .as_array()
                        .is_some_and(|m| m.iter().any(|v| v == "alipay"))
                    && rule["alipay_formats"].is_array()
                {
                    super::validate_alipay(&record, rule)?;
                    record["status"] = json!("confirmed");
                    record["policy_id"] = rule["id"].clone();
                }
            }
            let u = state.users.get_mut(actor).unwrap();
            check_version(u.version, input)?;
            record["version"] = json!(u.version + 1);
            let status = record["status"].clone();
            u.receipts.push(record);
            u.version += 1;
            json!({"id":generated,"version":u.version,"status":status,"notice":"资料版本已保存；草稿不能用于申请，格式检查不代表官方验证"})
        }
        "receipt_disable" => {
            let id = text(input, "id")?;
            let u = state.users.get_mut(actor).unwrap();
            if u.receipts.iter().any(|r| r["disables"] == id) {
                return Ok(json!({"version":u.version}));
            }
            check_version(u.version, input)?;
            // Append status event; original receipt and any snapshot never change.
            if !u
                .receipts
                .iter()
                .any(|r| r["id"] == id && r["method"].is_string())
            {
                return Err(Failure("NOT_FOUND", "资料不存在"));
            }
            u.receipts
                .push(json!({"id":generated,"disables":id,"status":"disabled","time":time}));
            u.version += 1;
            json!({"version":u.version})
        }
        _ => return Err(Failure("NOT_FOUND", "操作不存在")),
    };
    state.log(
        actor,
        action,
        result["id"].as_str().unwrap_or(actor),
        request,
        time,
    );
    if let Some(log) = state.audit.last_mut() {
        log["requirements"] = json!(["M17", "M18"]);
        log["object_version"] = result["version"].clone();
        log["declaration_version"] = json!("receipt-self-v1");
    }
    Ok(result)
}
