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
                    "请在申请详情关联本单范围及用途后授权查看收款快照",
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
