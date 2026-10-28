use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route(
            "/api/v1/community/read/attribution_admin",
            get(attribution_admin),
        )
        .route(
            "/api/v1/community/read/attribution_policies",
            get(attribution_policies),
        )
        .route("/api/v1/community/read/my_attribution", get(my_attribution))
        .route(
            "/api/v1/community/commands/policy_publish",
            post(policy_publish),
        )
        .route("/api/v1/community/commands/touch_bind", post(touch_bind))
        .route(
            "/api/v1/community/commands/attribution_dispute",
            post(attribution_dispute),
        )
        .route(
            "/api/v1/community/commands/dispute_resolve",
            post(dispute_resolve),
        )
}

async fn attribution_admin(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "attribution_admin", scope).await
}

async fn attribution_policies(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "attribution_policies", scope).await
}

async fn my_attribution(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "my_attribution", scope).await
}

async fn policy_publish(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "policy_publish", body).await
}

async fn touch_bind(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "touch_bind", body).await
}

async fn attribution_dispute(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "attribution_dispute", body).await
}

async fn dispute_resolve(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "dispute_resolve", body).await
}
