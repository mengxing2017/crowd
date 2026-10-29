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
    Router::new().route("/api/v1/identity/reauthenticate", post(handle))
}
async fn handle(
    State(app): State<Application>,
    headers: HeaderMap,
    body: std::result::Result<Json<Value>, JsonRejection>,
) -> Response {
    let request = http::request_id();
    let result = match http::parse(body) {
        Ok(v) => match crate::shared::text(&v, "password") {
            Ok(p) => {
                app.services
                    .staff_reauthentication
                    .execute(http::token(&headers), p, &request)
                    .await
            }
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    };
    http::response(result, &request)
}
