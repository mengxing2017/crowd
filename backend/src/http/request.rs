use crate::community::domain::Failure;
use axum::{Json, extract::rejection::JsonRejection, http::HeaderMap};
use serde_json::Value;

pub(crate) fn token(headers: &HeaderMap) -> &str {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("")
}
pub(crate) fn parse(body: Result<Json<Value>, JsonRejection>) -> Result<Value, Failure> {
    body.map(|v| v.0)
        .map_err(|_| Failure("INVALID_INPUT", "请求必须是合法 JSON 且未超过处理上限"))
}
