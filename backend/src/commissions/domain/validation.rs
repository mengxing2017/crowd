use crate::shared::{Failure, Result, invalid};
use serde_json::Value;
pub(crate) const MAX: i64 = 9_007_199_254_740_991;
pub(crate) fn require(ok: bool, message: &'static str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Failure("BUSINESS_CONFLICT", message))
    }
}
pub(crate) fn number(v: &Value, key: &str) -> Result<i64> {
    v[key]
        .as_i64()
        .filter(|n| n.unsigned_abs() <= MAX as u64)
        .ok_or(invalid("数字须为安全范围内的整数分"))
}
pub(crate) fn cents(v: &Value, key: &str) -> Result<i64> {
    let n = number(v, key)?;
    require(n > 0, "标记金额必须大于零")?;
    Ok(n)
}
pub(crate) fn version(v: &Value, input: &Value) -> Result<()> {
    crate::shared::check_version(v["version"].as_u64().unwrap_or(0), input)
}
