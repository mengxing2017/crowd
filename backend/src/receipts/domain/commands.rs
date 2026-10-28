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
            let mut record = json!({"id":generated,"method":method,"status":"confirmed","time":time,"owner":actor,"official_verified":false});
            if method == "wechat" {
                record["file_id"] = json!(text(input, "file_id")?);
            } else {
                // Supported formats remain unconfirmed; accepting opaque drafts avoids declaring validity.
                record["account"] = json!(text(input, "account")?);
                record["name"] = input["name"].clone();
                record["status"] = json!("draft");
            }
            let u = state.users.get_mut(actor).unwrap();
            check_version(u.version, input)?;
            record["version"] = json!(u.version + 1);
            u.receipts.push(record);
            u.version += 1;
            json!({"id":generated,"version":u.version,"notice":"资料已保存，不代表官方账户验证；支付宝支持形式待确认"})
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
    state.log(actor, action, actor, request, time);
    Ok(result)
}
