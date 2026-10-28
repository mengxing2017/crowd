use crate::{
    Application,
    http::{parse, request_id, response},
    platform::domain::Failure,
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::HeaderMap,
    response::Response,
};
use serde_json::Value;

pub(super) async fn touch(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let result = match (
        headers
            .get("idempotency-key")
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.is_empty() && s.len() <= 128),
        parse(body),
    ) {
        (Some(key), Ok(v)) => app.services.touches.execute(&v, key, &r).await,
        _ => Err(Failure(
            "INVALID_INPUT",
            "访问需合法 JSON 及 Idempotency-Key",
        )),
    };
    response(result, &r)
}
