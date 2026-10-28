use crate::{
    Application,
    http::{parse, request_id, response},
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    response::Response,
};
use serde_json::Value;

pub(crate) async fn staff_login(
    State(app): State<Application>,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let result = match parse(body) {
        Ok(input) => app.services.staff_login.execute(&input, &r).await,
        Err(e) => Err(e),
    };
    response(result, &r)
}
