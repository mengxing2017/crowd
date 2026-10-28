use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/command_result", get(command_result))
        .route("/api/v1/community/commands/logout", post(logout))
        .route("/api/v1/community/commands/web_ticket", post(web_ticket))
}

async fn command_result(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "command_result", scope).await
}

async fn logout(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "logout", body).await
}

async fn web_ticket(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "web_ticket", body).await
}
