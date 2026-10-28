use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route(
            "/api/v1/community/commands/review_claim",
            post(review_claim),
        )
        .route(
            "/api/v1/community/commands/review_decide",
            post(review_decide),
        )
        .route(
            "/api/v1/community/commands/review_verify",
            post(review_verify),
        )
        .route(
            "/api/v1/community/commands/review_batch_preview",
            post(review_batch_preview),
        )
        .route(
            "/api/v1/community/commands/review_batch",
            post(review_batch),
        )
}

async fn review_claim(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "review_claim", body).await
}

async fn review_decide(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "review_decide", body).await
}

async fn review_verify(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "review_verify", body).await
}

async fn review_batch_preview(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "review_batch_preview", body).await
}

async fn review_batch(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "review_batch", body).await
}
