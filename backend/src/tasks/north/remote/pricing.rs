use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/commands/price_save", post(price_save))
        .route(
            "/api/v1/community/commands/price_review",
            post(price_review),
        )
}

async fn price_save(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "price_save", body).await
}

async fn price_review(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "price_review", body).await
}
