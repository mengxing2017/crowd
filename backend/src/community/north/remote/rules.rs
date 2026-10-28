use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/rules", get(rules))
        .route("/api/v1/community/commands/rule_save", post(rule_save))
        .route(
            "/api/v1/community/commands/rule_publish",
            post(rule_publish),
        )
        .route(
            "/api/v1/community/commands/rule_disable",
            post(rule_disable),
        )
}

async fn rules(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "rules", scope).await
}

async fn rule_save(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "rule_save", body).await
}

async fn rule_publish(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "rule_publish", body).await
}

async fn rule_disable(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "rule_disable", body).await
}
