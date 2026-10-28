use super::{parse, request_id, response, token};
use crate::{Application, community::domain::Failure};
use axum::{Json, extract::rejection::JsonRejection, http::HeaderMap, response::Response};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub(crate) struct Scope {
    pub(crate) target: Option<String>,
}

pub(crate) async fn read(
    app: &Application,
    headers: &HeaderMap,
    section: &str,
    scope: Scope,
) -> Response {
    let r = request_id();
    response(
        app.community
            .read(token(headers), section, scope.target.as_deref(), &r)
            .await,
        &r,
    )
}
pub(crate) async fn command(
    app: &Application,
    headers: &HeaderMap,
    action: &str,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty() && s.len() <= 128);
    let result = match (key, parse(body)) {
        (Some(key), Ok(input)) => {
            app.community
                .command(token(headers), action, &input, key, &r)
                .await
        }
        (None, _) => Err(Failure("INVALID_INPUT", "缺少有效 Idempotency-Key")),
        (_, Err(e)) => Err(e),
    };
    response(result, &r)
}
