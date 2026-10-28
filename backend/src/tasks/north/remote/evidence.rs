use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route(
            "/api/v1/community/commands/evidence_discard",
            post(evidence_discard),
        )
        .route(
            "/api/v1/community/commands/evidence_save",
            post(evidence_save),
        )
        .route(
            "/api/v1/community/commands/evidence_preview",
            post(evidence_preview),
        )
        .route(
            "/api/v1/community/commands/evidence_submit",
            post(evidence_submit),
        )
        .route(
            "/api/v1/community/commands/evidence_withdraw",
            post(evidence_withdraw),
        )
        .route(
            "/api/v1/community/commands/evidence_reveal",
            post(evidence_reveal),
        )
}

async fn evidence_discard(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "evidence_discard", body).await
}

async fn evidence_save(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "evidence_save", body).await
}

async fn evidence_preview(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "evidence_preview", body).await
}

async fn evidence_submit(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "evidence_submit", body).await
}

async fn evidence_withdraw(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "evidence_withdraw", body).await
}

async fn evidence_reveal(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "evidence_reveal", body).await
}
