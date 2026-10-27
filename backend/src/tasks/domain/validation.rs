use crate::community::domain::{Failure, Result, invalid};
use serde_json::Value;

pub(super) fn number(v: &Value, field: &str) -> Result<i64> {
    v[field]
        .as_i64()
        .filter(|v| v.unsigned_abs() <= 9_007_199_254_740_991)
        .ok_or(invalid("金额、数量和时间须为安全整数"))
}
pub(super) fn positive(v: &Value, field: &str) -> Result<i64> {
    number(v, field).and_then(|n| {
        if n > 0 {
            Ok(n)
        } else {
            Err(invalid("时限和数量必须大于零"))
        }
    })
}
pub(super) fn conflict() -> Failure {
    Failure("VERSION_CONFLICT", "对象或状态已改变，请刷新后重试")
}
pub(super) fn missing() -> Failure {
    Failure("NOT_FOUND", "任务业务对象不存在")
}
pub(super) fn require(condition: bool, message: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}
pub(super) fn version(v: &Value) -> u64 {
    v["version"].as_u64().unwrap_or(0)
}
pub(super) fn array<'a>(v: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    v[field].as_array().ok_or(invalid("数组字段缺失"))
}
pub(super) fn next_time(now: i64, seconds: i64) -> Result<i64> {
    now.checked_add(seconds)
        .filter(|n| *n <= 9_007_199_254_740_991)
        .ok_or(invalid("时间超出范围"))
}
pub(super) fn active(status: &Value) -> bool {
    matches!(
        status.as_str(),
        Some(
            "claimed"
                | "in_progress"
                | "supplement"
                | "submitted"
                | "reviewing"
                | "awaiting_verification"
                | "paused"
        )
    )
}
