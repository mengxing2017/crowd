use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route(
            "/api/v1/community/commands/template_save",
            post(template_save),
        )
        .route(
            "/api/v1/community/commands/template_review",
            post(template_review),
        )
        .route(
            "/api/v1/community/commands/template_disable",
            post(template_disable),
        )
}

async fn template_save(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "template_save", body).await
}

async fn template_review(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "template_review", body).await
}

async fn template_disable(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "template_disable", body).await
}
