use super::masking::mask_receipts;
use crate::receipts::south::port::DomainPort;
use crate::shared::{Failure, Result};
use serde_json::{Value, json};
pub(crate) fn read<P: DomainPort + ?Sized>(
    state: &P,
    actor: &str,
    section: &str,
    target: &str,
) -> Result<Value> {
    match section {
        "receipts" => {
            if target != actor {
                return Err(Failure(
                    "FORBIDDEN",
                    "请在申请详情关联本单范围及用途后授权查看收款快照",
                ));
            }
            Ok(json!(mask_receipts(&state.user(actor)?.receipts)))
        }
        _ => Err(Failure("NOT_FOUND", "接口不存在")),
    }
}
