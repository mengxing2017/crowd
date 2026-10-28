use crate::{
    Application,
    http::{self, Scope},
};
use axum::{
    Json,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::HeaderMap,
    response::Response,
};
use serde_json::Value;

pub(crate) async fn read(
    State(app): State<Application>,
    Path(section): Path<String>,
    Query(scope): Query<Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, &section, scope).await
}
pub(crate) async fn command(
    State(app): State<Application>,
    Path(action): Path<String>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, &action, body).await
}
