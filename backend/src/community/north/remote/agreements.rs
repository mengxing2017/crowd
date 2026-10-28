use crate::{
    Application,
    http::{request_id, response},
};
use axum::{extract::State, response::Response};

pub(super) async fn agreements(State(app): State<Application>) -> Response {
    let r = request_id();
    response(app.services.agreements.execute().await, &r)
}
