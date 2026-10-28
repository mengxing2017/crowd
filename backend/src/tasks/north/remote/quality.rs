use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/task_changes", get(task_changes))
        .route(
            "/api/v1/community/commands/quality_inspect",
            post(quality_inspect),
        )
}

async fn task_changes(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "task_changes", scope).await
}

async fn quality_inspect(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "quality_inspect", body).await
}
