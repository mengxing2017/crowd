use crate::Application;
use crate::http;
use axum::Router;
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};

pub(super) fn routes() -> Router<Application> {
    Router::new().route("/api/v1/community/read/audit", get(audit))
}

async fn audit(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "audit", scope).await
}
