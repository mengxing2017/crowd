use crate::{
    Application,
    community::south::adapter::wechat::WechatIdentity,
    http::{parse, request_id, response},
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    response::Response,
};
use serde_json::Value;

pub(super) async fn login(
    State(app): State<Application>,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let result = match parse(body) {
        Ok(input) => app.community.login(&WechatIdentity, &input, &r).await,
        Err(e) => Err(e),
    };
    response(result, &r)
}
