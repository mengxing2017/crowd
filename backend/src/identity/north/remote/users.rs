use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(crate) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/me", get(me))
        .route("/api/v1/community/read/users", get(users))
        .route("/api/v1/community/commands/profile", post(profile))
        .route("/api/v1/community/commands/phone", post(phone))
        .route("/api/v1/community/commands/access", post(access))
        .route("/api/v1/community/commands/restriction", post(restriction))
}

async fn me(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "me", scope).await
}

async fn users(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "users", scope).await
}

async fn profile(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "profile", body).await
}

async fn phone(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "phone", body).await
}

async fn access(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "access", body).await
}

async fn restriction(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "restriction", body).await
}
