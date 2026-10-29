use crate::privacy::south::port::DomainPort;
use crate::shared::{Failure, Result, number, require, text};
use serde_json::Value;
fn referenced(v: &Value, id: &str) -> bool {
    match v {
        Value::String(s) => s == id,
        Value::Array(a) => a.iter().any(|v| referenced(v, id)),
        Value::Object(m) => m.values().any(|v| referenced(v, id)),
        _ => false,
    }
}
pub(crate) fn cleanup_candidate<P: DomainPort + ?Sized>(s: &P, i: &Value) -> Result<i64> {
    let id = text(i, "id")?;
    text(i, "reason")?;
    let purpose = text(i, "purpose")?;
    let key = match purpose {
        "wechat_receipt" => "wechat_receipt_seconds",
        "task_evidence" => "task_evidence_seconds",
        "settlement_voucher" => "settlement_voucher_seconds",
        "support_attachment" => "support_attachment_seconds",
        _ => return Err(Failure("INVALID_INPUT", "不支持此文件用途")),
    };
    let policy =
        super::commands::policy(s).ok_or(Failure("POLICY_REQUIRED", "图片保留政策尚未发布"))?;
    require(
        !referenced(&s.facts(), id),
        "文件仍被资料、任务、申请或争议证据引用，禁止清理",
    )?;
    number(&policy["content"], key)
}
