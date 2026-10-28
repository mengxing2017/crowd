use crate::Application;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
pub fn routes() -> Router<Application> {
    Router::new()
        .merge(crate::commissions::remote::routes())
        .merge(crate::withdrawals::remote::routes())
        .merge(crate::settlements::remote::routes())
        .merge(crate::identity::remote::routes())
        .merge(crate::receipts::remote::routes())
        .merge(crate::promotion::remote::routes())
        .merge(crate::governance::remote::routes())
        .merge(crate::audit::remote::routes())
        .merge(crate::support::remote::routes())
        .merge(crate::files::remote::routes())
        .merge(crate::catalog::remote::routes())
        .merge(crate::tasks::remote::routes())
        .route(
            "/api/v1/community/read/{section}",
            get(crate::platform::north::remote::compatibility::read),
        )
        .route(
            "/api/v1/community/commands/{action}",
            post(crate::platform::north::remote::compatibility::command),
        )
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
}
