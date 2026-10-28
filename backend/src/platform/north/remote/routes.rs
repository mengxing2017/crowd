use crate::Application;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
pub fn routes() -> Router<Application> {
    Router::new()
        .merge(crate::identity::north::remote::routes())
        .merge(crate::receipts::north::remote::routes())
        .merge(crate::promotion::north::remote::routes())
        .merge(crate::governance::north::remote::routes())
        .merge(crate::audit::north::remote::routes())
        .merge(crate::support::north::remote::routes())
        .merge(crate::files::north::remote::routes())
        .merge(crate::catalog::north::remote::routes())
        .merge(crate::tasks::north::remote::routes())
        .route(
            "/api/v1/community/read/{section}",
            get(super::compatibility::read),
        )
        .route(
            "/api/v1/community/commands/{action}",
            post(super::compatibility::command),
        )
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
}
