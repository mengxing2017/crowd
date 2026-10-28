use super::masking::mask_receipt;
use crate::platform::domain::State;
use crate::shared::{Failure, Result};
use serde_json::{Value, json};
pub(crate) fn read(state: &State, actor: &str, section: &str, target: &str) -> Result<Value> {
    match section {
        "receipts" => {
            if target != actor {
                return Err(Failure(
                    "FORBIDDEN",
                    "后台查看收款资料必须关联申请范围与用途，申请服务尚未接入",
                ));
            }
            Ok(json!(
                state
                    .user(actor)?
                    .receipts
                    .iter()
                    .map(mask_receipt)
                    .collect::<Vec<_>>()
            ))
        }
        _ => Err(Failure("NOT_FOUND", "接口不存在")),
    }
}
