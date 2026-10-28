use crate::platform::domain::Failure;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};

static REQUEST: AtomicU64 = AtomicU64::new(1);
pub(crate) fn request_id() -> String {
    format!(
        "{:x}-{:x}-{:x}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        REQUEST.fetch_add(1, Ordering::Relaxed)
    )
}
pub(crate) fn response(result: Result<Value, Failure>, request: &str) -> Response {
    let (status, value) = match result {
        Ok(v) => (StatusCode::OK, json!({"data":v,"request_id":request})),
        Err(Failure(code, message)) => {
            let status = match code {
                "SESSION_EXPIRED" | "AUTH_INVALID" => StatusCode::UNAUTHORIZED,
                "FORBIDDEN" => StatusCode::FORBIDDEN,
                "NOT_FOUND" => StatusCode::NOT_FOUND,
                "VERSION_CONFLICT" | "IDEMPOTENCY_CONFLICT" => StatusCode::CONFLICT,
                "SERVICE_UNAVAILABLE"
                | "IDENTITY_UNAVAILABLE"
                | "DEPENDENCY_UNAVAILABLE"
                | "AGREEMENT_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::UNPROCESSABLE_ENTITY,
            };
            (
                status,
                json!({"error":{"code":code,"message":message,"request_id":request}}),
            )
        }
    };
    (
        status,
        [
            ("cache-control", "no-store"),
            ("x-content-type-options", "nosniff"),
        ],
        Json(value),
    )
        .into_response()
}
