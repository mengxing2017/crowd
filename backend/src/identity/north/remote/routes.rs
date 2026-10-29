use crate::Application;
use axum::Router;
use axum::routing::post;
pub fn routes() -> Router<Application> {
    Router::new()
        .merge(super::reauthenticate::routes())
        .merge(super::security_policy_publish::routes())
        .merge(super::temporary_grant_request::routes())
        .merge(super::temporary_grant_review::routes())
        .merge(super::temporary_grant_revoke::routes())
        .merge(super::security::routes())
        .route("/api/v1/community/login", post(super::login::login))
        .route(
            "/api/v1/community/admin/login",
            post(super::staff_login::staff_login),
        )
        .route("/api/v1/community/session", post(super::session::redeem))
        .merge(super::users::routes())
        .merge(super::session_commands::routes())
}
