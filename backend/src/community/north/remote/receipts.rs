use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/receipts", get(receipts))
        .route("/api/v1/community/commands/receipt", post(receipt))
        .route(
            "/api/v1/community/commands/receipt_disable",
            post(receipt_disable),
        )
        .route(
            "/api/v1/community/commands/receipt_reveal",
            post(receipt_reveal),
        )
}

async fn receipts(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "receipts", scope).await
}

async fn receipt(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "receipt", body).await
}

async fn receipt_disable(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "receipt_disable", body).await
}

async fn receipt_reveal(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "receipt_reveal", body).await
}
