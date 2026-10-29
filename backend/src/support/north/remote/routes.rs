use crate::Application;
use axum::Router;
pub fn routes() -> Router<Application> {
    Router::new()
        .merge(super::ticket_create::routes())
        .merge(super::ticket_assign::routes())
        .merge(super::ticket_reply::routes())
        .merge(super::ticket_close::routes())
        .merge(super::ticket_reopen::routes())
        .merge(super::support_policy::routes())
        .merge(super::tickets::routes())
        .merge(super::support_statistics::routes())
        .merge(super::support_policy_read::routes())
        .merge(super::cases::routes())
}
