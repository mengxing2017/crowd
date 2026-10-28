use crate::Application;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router<Application> {
    Router::new()
        .route("/api/v1/catalog", get(super::directory::catalog))
        .route("/api/v1/catalog/touches", post(super::touches::touch))
        .route(
            "/api/v1/catalog/images/{id}",
            get(super::images::product_image),
        )
        .merge(super::products::routes())
        .merge(super::categories::routes())
        .merge(super::attribution::routes())
}
