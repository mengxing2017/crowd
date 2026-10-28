use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(crate) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/cases", get(cases))
        .route("/api/v1/community/commands/correction", post(correction))
        .route("/api/v1/community/commands/closure", post(closure))
        .route("/api/v1/community/commands/merge", post(merge))
}

async fn cases(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "cases", scope).await
}

async fn correction(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "correction", body).await
}

async fn closure(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "closure", body).await
}

async fn merge(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "merge", body).await
}
