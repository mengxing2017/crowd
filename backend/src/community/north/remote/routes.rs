use crate::Application;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

pub fn routes() -> Router<Application> {
    Router::new()
        .merge(crate::catalog::north::remote::routes())
        .merge(crate::tasks::north::remote::routes())
        .route(
            "/api/v1/community/agreements",
            get(super::agreements::agreements),
        )
        .route("/api/v1/community/login", post(super::login::login))
        .route(
            "/api/v1/community/admin/login",
            post(super::staff_login::staff_login),
        )
        .route("/api/v1/community/session", post(super::session::redeem))
        .route(
            "/api/v1/community/resolve/{code}",
            get(super::promotions::resolve),
        )
        .route("/api/v1/community/qr/{code}", get(super::promotions::qr))
        .route(
            "/api/v1/community/files/{id}",
            get(super::files::file).post(super::files::upload),
        )
        .route(
            "/api/v1/community/read/{section}",
            get(super::compatibility::read),
        )
        .route(
            "/api/v1/community/commands/{action}",
            post(super::compatibility::command),
        )
        .merge(super::users::routes())
        .merge(super::receipts::routes())
        .merge(super::rules::routes())
        .merge(super::audit::routes())
        .merge(super::cases::routes())
        .merge(super::session_commands::routes())
        .merge(super::codes::routes())
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
}
