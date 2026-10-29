use crate::Application;
use axum::Router;
pub fn routes() -> Router<Application> {
    Router::new()
        .merge(super::notification_read::routes())
        .merge(super::notification_consent::routes())
        .merge(super::notification_template::routes())
        .merge(super::notification_delivery::routes())
        .merge(super::messages::routes())
        .merge(super::notification_templates::routes())
        .merge(super::notification_todos::routes())
}
