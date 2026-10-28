use crate::Application;
use crate::http;
use axum::Router;
use axum::{Json, extract::rejection::JsonRejection, routing::post};
use axum::{extract::Query, routing::get};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde_json::Value;

pub(super) fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/community/read/catalog_admin", get(catalog_admin))
        .route(
            "/api/v1/community/read/catalog_notifications",
            get(catalog_notifications),
        )
        .route(
            "/api/v1/community/commands/product_save",
            post(product_save),
        )
        .route(
            "/api/v1/community/commands/product_submit",
            post(product_submit),
        )
        .route(
            "/api/v1/community/commands/product_review",
            post(product_review),
        )
        .route(
            "/api/v1/community/commands/product_offline",
            post(product_offline),
        )
        .route(
            "/api/v1/community/commands/product_archive",
            post(product_archive),
        )
        .route(
            "/api/v1/community/commands/product_rollback",
            post(product_rollback),
        )
}

async fn catalog_admin(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "catalog_admin", scope).await
}

async fn catalog_notifications(
    State(app): State<Application>,
    Query(scope): Query<http::Scope>,
    headers: HeaderMap,
) -> Response {
    http::read(&app, &headers, "catalog_notifications", scope).await
}

async fn product_save(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "product_save", body).await
}

async fn product_submit(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "product_submit", body).await
}

async fn product_review(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "product_review", body).await
}

async fn product_offline(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "product_offline", body).await
}

async fn product_archive(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "product_archive", body).await
}

async fn product_rollback(
    State(app): State<Application>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Response {
    http::command(&app, &headers, "product_rollback", body).await
}
