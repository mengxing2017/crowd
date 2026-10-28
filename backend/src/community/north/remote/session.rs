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

pub(super) async fn redeem(
    State(app): State<Application>,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let result = match parse(body) {
        Ok(v) => match crate::community::domain::text(&v, "ticket") {
            Ok(ticket) => app.community.repo.redeem(ticket).await,
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    };
    response(result, &r)
}
