use crate::{Application, http};
use axum::{
    Json, Router, extract::State, extract::rejection::JsonRejection, http::HeaderMap,
    response::Response, routing::post,
};
use serde_json::Value;
pub(crate) fn routes() -> Router<Application> {
    Router::new().route("/api/v1/community/commands/adjustment_locate", post(handle))
}
async fn handle(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "adjustment_locate", body).await
}
