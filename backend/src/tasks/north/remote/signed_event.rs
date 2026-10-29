use crate::{Application, http};
use axum::{
    Router, body::Bytes, extract::State, http::HeaderMap, response::Response, routing::post,
};
pub(crate) fn routes() -> Router<Application> {
    Router::new().route("/api/v1/task-events", post(handle))
}
async fn handle(State(app): State<Application>, headers: HeaderMap, body: Bytes) -> Response {
    let r = http::request_id();
    let source = headers
        .get("x-task-source")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let signature = headers
        .get("x-task-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    http::response(
        app.services
            .task_events
            .execute(source, signature, &body, &r)
            .await,
        &r,
    )
}
