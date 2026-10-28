use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route(
            "/api/v1/community/commands/task_repair_propose",
            post(task_repair_propose),
        )
        .route(
            "/api/v1/community/commands/task_repair_review",
            post(task_repair_review),
        )
        .route(
            "/api/v1/community/commands/review_revoke_propose",
            post(review_revoke_propose),
        )
        .route(
            "/api/v1/community/commands/review_revoke_review",
            post(review_revoke_review),
        )
}

async fn task_repair_propose(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "task_repair_propose", body).await
}

async fn task_repair_review(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "task_repair_review", body).await
}

async fn review_revoke_propose(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "review_revoke_propose", body).await
}

async fn review_revoke_review(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "review_revoke_review", body).await
}
