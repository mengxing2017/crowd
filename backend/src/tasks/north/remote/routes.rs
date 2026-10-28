use crate::Application;
use axum::Router;

pub fn routes() -> Router<Application> {
    Router::new()
        .merge(super::eligibility::routes())
        .merge(super::rules::routes())
        .merge(super::pricing::routes())
        .merge(super::templates::routes())
        .merge(super::instances::routes())
        .merge(super::evidence::routes())
        .merge(super::review::routes())
        .merge(super::corrections::routes())
        .merge(super::quality::routes())
}
