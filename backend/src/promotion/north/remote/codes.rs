use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(crate) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/codes", get(codes))
        .route("/api/v1/community/read/managed_codes", get(managed_codes))
        .route("/api/v1/community/commands/code_create", post(code_create))
        .route("/api/v1/community/commands/code_rotate", post(code_rotate))
}

async fn codes(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "codes", scope).await
}

async fn managed_codes(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "managed_codes", scope).await
}

async fn code_create(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "code_create", body).await
}

async fn code_rotate(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "code_rotate", body).await
}
