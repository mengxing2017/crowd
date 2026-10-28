use crate::{Application, http};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};
pub(super) async fn handle(
    State(app): State<Application>,
    headers: HeaderMap,
    Query(scope): Query<http::Scope>,
) -> Response {
    http::read(&app, &headers, "commission_summary", scope).await
}
