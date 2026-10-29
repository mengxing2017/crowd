use crate::{Application, http};
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{Router, extract::State, http::HeaderMap, response::Response};
use serde_json::Value;
pub(crate) fn routes() -> Router<Application> {
    Router::new().route("/api/v1/community/commands/difference_assign", post(handle))
}
async fn handle(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "difference_assign", body).await
}
