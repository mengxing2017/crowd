use crate::{Application, http};
use axum::{
    Json, Router,
    extract::{Query, State, rejection::JsonRejection},
    http::HeaderMap,
    response::Response,
    routing::{get, post},
};
use serde_json::Value;
pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/eligibility", get(eligibility))
        .route(
            "/api/v1/community/commands/eligibility_verify",
            post(eligibility_verify),
        )
}
async fn eligibility(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "eligibility", scope).await
}

async fn eligibility_verify(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "eligibility_verify", body).await
}
