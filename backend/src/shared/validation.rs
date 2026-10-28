use serde_json::Value;
#[derive(Debug, Clone)]
pub struct Failure(pub &'static str, pub &'static str);
pub type Result<T> = std::result::Result<T, Failure>;
pub fn invalid(message: &'static str) -> Failure {
    Failure("INVALID_INPUT", message)
}
pub fn text<'a>(v: &'a Value, field: &str) -> Result<&'a str> {
    v[field]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 4096)
        .ok_or(invalid("必填字段缺失或过长"))
}
pub fn check_version(version: u64, input: &Value) -> Result<()> {
    if input["version"].as_u64() != Some(version) {
        Err(Failure("VERSION_CONFLICT", "版本已变更，请重新查询"))
    } else {
        Ok(())
    }
}
/// Integer-only display; no floating point participates in business amount decisions.
pub fn display_cents(cents: i64) -> String {
    let value = i128::from(cents);
    let absolute = value.abs();
    format!(
        "{}{}.{:02}",
        if value < 0 { "-" } else { "" },
        absolute / 100,
        absolute % 100
    )
}

pub fn validate_fields(input: &Value, fields: &[&str]) -> Result<()> {
    let object = input.as_object().ok_or(invalid("请求内容必须为对象"))?;
    if object.keys().any(|key| !fields.contains(&key.as_str())) {
        return Err(invalid("请求包含当前操作不接受的字段"));
    }
    Ok(())
}
