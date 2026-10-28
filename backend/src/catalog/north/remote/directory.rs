use crate::{
    Application,
    community::domain::Failure,
    http::{request_id, response},
};
use axum::{
    extract::{Query, State},
    response::Response,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub(super) struct CatalogQuery {
    keyword: Option<String>,
    category: Option<String>,
    min_cents: Option<i64>,
    max_cents: Option<i64>,
    sort: Option<String>,
    task_status: Option<String>,
}
pub(super) async fn catalog(
    State(app): State<Application>,
    query: Result<Query<CatalogQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let r = request_id();
    let result=match query {Ok(Query(q))=>app.community.public_catalog(&json!({"keyword":q.keyword,"category":q.category,"min_cents":q.min_cents,"max_cents":q.max_cents,"sort":q.sort,"task_status":q.task_status})).await, Err(_)=>Err(Failure("INVALID_INPUT","目录筛选参数无效"))};
    response(result, &r)
}
