use crate::Application;
use axum::Router;
pub fn routes() -> Router<Application> {
    Router::new()
        .merge(super::report_create::routes())
        .merge(super::export_request::routes())
        .merge(super::reconciliation_run::routes())
        .merge(super::difference_assign::routes())
        .merge(super::difference_close::routes())
        .merge(super::voucher_check::routes())
        .merge(super::reports::routes())
        .merge(super::report_detail::routes())
        .merge(super::export_jobs::routes())
        .merge(super::export_download::routes())
        .merge(super::differences::routes())
}
