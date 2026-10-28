use crate::Application;
use axum::Router;
use axum::routing::get;
pub fn routes() -> Router<Application> {
    Router::new()
        .route(
            "/api/v1/community/resolve/{code}",
            get(super::promotions::resolve),
        )
        .route("/api/v1/community/qr/{code}", get(super::promotions::qr))
        .merge(super::codes::routes())
}
