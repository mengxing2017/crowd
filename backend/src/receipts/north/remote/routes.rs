use crate::Application;
use axum::Router;
pub fn routes() -> Router<Application> {
    Router::new().merge(super::receipts::routes())
}
