use crate::{Application, http};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::HeaderMap,
    response::Response,
};
use serde_json::Value;
pub(super) async fn handle(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "commission_adjust_review", body).await
}
