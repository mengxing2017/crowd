use crate::Application;
use axum::Router;
use axum::routing::get;
pub fn routes() -> Router<Application> {
    Router::new().route(
        "/api/v1/community/files/{id}",
        get(super::files::file).post(super::files::upload),
    )
}
