use crate::{Application, http};
use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::HeaderMap,
    response::Response,
    routing::post,
};
use serde_json::Value;
pub(crate) fn routes() -> Router<Application> {
    Router::new().route("/api/v1/community/commands/receipt_cleanup", post(cleanup))
}
async fn cleanup(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "receipt_cleanup", body).await
}
