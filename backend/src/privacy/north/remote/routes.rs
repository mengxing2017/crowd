use crate::Application;
use axum::Router;
pub fn routes() -> Router<Application> {
    Router::new()
        .merge(super::privacy_file_cleanup::routes())
        .merge(super::privacy_policy_publish::routes())
        .merge(super::privacy_copy_request::routes())
        .merge(super::privacy_close_request::routes())
        .merge(super::privacy_close_review::routes())
        .merge(super::privacy_repair_preview::routes())
        .merge(super::privacy_repair_review::routes())
        .merge(super::privacy_policy::routes())
        .merge(super::privacy_requests::routes())
        .merge(super::privacy_copy::routes())
        .merge(super::privacy_repairs::routes())
}
