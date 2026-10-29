use crate::Application;
use axum::Router;
pub fn routes() -> Router<Application> {
    Router::new()
        .merge(super::campaign_save::routes())
        .merge(super::campaign_transfer::routes())
        .merge(super::campaign_archive::routes())
        .merge(super::campaigns::routes())
        .merge(super::campaign_report::routes())
}
