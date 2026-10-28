use crate::Application;
use axum::Router;
use axum::routing::get;
pub fn routes() -> Router<Application> {
    Router::new()
        .route(
            "/api/v1/community/agreements",
            get(super::agreements::agreements),
        )
        .merge(super::rules::routes())
}
