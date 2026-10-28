use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/my_claims", get(my_claims))
        .route("/api/v1/community/read/my_tasks", get(my_tasks))
        .route("/api/v1/community/read/task_queue", get(task_queue))
        .route("/api/v1/community/read/task_detail", get(task_detail))
        .route("/api/v1/community/commands/task_claim", post(task_claim))
        .route("/api/v1/community/commands/task_start", post(task_start))
        .route("/api/v1/community/commands/task_step", post(task_step))
        .route(
            "/api/v1/community/commands/task_abandon",
            post(task_abandon),
        )
        .route("/api/v1/community/commands/task_pause", post(task_pause))
        .route("/api/v1/community/commands/task_resume", post(task_resume))
        .route("/api/v1/community/commands/task_expire", post(task_expire))
}

async fn my_claims(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "my_claims", scope).await
}

async fn my_tasks(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "my_tasks", scope).await
}

async fn task_queue(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "task_queue", scope).await
}

async fn task_detail(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "task_detail", scope).await
}

async fn task_claim(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "task_claim", body).await
}

async fn task_start(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "task_start", body).await
}

async fn task_step(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "task_step", body).await
}

async fn task_abandon(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "task_abandon", body).await
}

async fn task_pause(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "task_pause", body).await
}

async fn task_resume(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "task_resume", body).await
}

async fn task_expire(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "task_expire", body).await
}
