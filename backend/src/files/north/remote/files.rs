use crate::{
    Application,
    http::{request_id, response, token},
    shared::Failure,
};
use axum::{
    body::Bytes,
    extract::{Path, Query, State, rejection::BytesRejection},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

pub(crate) async fn upload(
    State(app): State<Application>,
    Path(purpose): Path<String>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    let r = request_id();
    let result = match body {
        Ok(bytes) => {
            app.services
                .uploads
                .execute(token(&headers), &purpose, bytes.to_vec(), &r)
                .await
        }
        Err(_) => Err(Failure(
            "INVALID_INPUT",
            "图片请求超过服务处理上限或读取失败",
        )),
    };
    response(result, &r)
}

#[derive(Deserialize, Default)]
pub(crate) struct FileQuery {
    reason: Option<String>,
}

pub(crate) async fn file(
    State(app): State<Application>,
    Path(id): Path<String>,
    headers: HeaderMap,
    query: Result<Query<FileQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let r = request_id();
    let query = match query {
        Ok(Query(query)) => query,
        Err(_) => return response(Err(Failure("INVALID_INPUT", "图片查看用途参数无效")), &r),
    };
    let result = if let Some(reason) = query.reason {
        app.services
            .sensitive_files
            .execute(token(&headers), &id, &reason, &r)
            .await
    } else {
        app.services
            .private_files
            .execute(token(&headers), &id, &r)
            .await
    };
    match result {
        Ok((mime, bytes)) => (
            [
                ("content-type", mime),
                ("cache-control", "no-store".into()),
                ("x-content-type-options", "nosniff".into()),
            ],
            bytes,
        )
            .into_response(),
        Err(e) => response(Err(e), &r),
    }
}
