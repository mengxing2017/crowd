use crate::{
    Application,
    http::{parse, request_id, response},
    identity::south::port::wechat::WechatIdentity,
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    response::Response,
};
use serde_json::Value;

pub(crate) async fn login(
    State(app): State<Application>,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    let r = request_id();
    let result = match parse(body) {
        Ok(input) => {
            app.services
                .wechat_login
                .execute(&WechatIdentity, &input, &r)
                .await
        }
        Err(e) => Err(e),
    };
    response(result, &r)
}
