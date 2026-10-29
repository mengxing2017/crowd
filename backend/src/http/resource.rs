use super::{parse, request_id, response, token};
use crate::{Application, shared::Failure};
use axum::{Json, extract::rejection::JsonRejection, http::HeaderMap, response::Response};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub(crate) struct Scope {
    pub(crate) target: Option<String>,
    pub(crate) limit: Option<usize>,
    pub(crate) cursor: Option<String>,
}

pub(crate) async fn read(
    app: &Application,
    headers: &HeaderMap,
    section: &str,
    scope: Scope,
) -> Response {
    let r = request_id();
    response(
        app.services
            .queries
            .execute(token(headers), section, scope.target.as_deref(), &r)
            .await
            .and_then(|v| super::pagination::paginate(v, scope.limit, scope.cursor.as_deref())),
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
            app.services
                .commands
                .execute(token(headers), action, &input, key, &r)
                .await
        }
        (None, _) => Err(Failure("INVALID_INPUT", "缺少有效 Idempotency-Key")),
        (_, Err(e)) => Err(e),
    };
    let details = if matches!(action, "withdrawal_apply" | "withdrawal_preview")
        && result
            .as_ref()
            .err()
            .is_some_and(|e| matches!(e.0, "VERSION_CONFLICT" | "INSUFFICIENT_AVAILABLE"))
    {
        app.services
            .withdrawal_failure_context
            .execute(token(headers), &r)
            .await
            .ok()
    } else {
        None
    };
    super::envelope::response_with_details(result, &r, details)
}
