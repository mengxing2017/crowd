use crate::{Application, http};
use axum::{Router, extract::State, http::HeaderMap, response::Response};
use axum::{extract::Query, routing::get};
pub(crate) fn routes() -> Router<Application> {
    Router::new().route("/api/v1/community/read/notification_templates", get(handle))
}
async fn handle(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "notification_templates", scope).await
}
