use crate::Application;
use axum::{
    Router,
    routing::{get, post},
};
pub fn routes() -> Router<Application> {
    Router::new()
        .route(
            "/api/v1/community/read/withdrawal_rules",
            get(super::withdrawal_rules::handle),
        )
        .route(
            "/api/v1/community/read/my_withdrawals",
            get(super::my_withdrawals::handle),
        )
        .route(
            "/api/v1/community/read/withdrawal_queue",
            get(super::withdrawal_queue::handle),
        )
        .route(
            "/api/v1/community/read/withdrawal_detail",
            get(super::withdrawal_detail::handle),
        )
        .route(
            "/api/v1/community/commands/withdrawal_rule_propose",
            post(super::withdrawal_rule_propose::handle),
        )
        .route(
            "/api/v1/community/commands/withdrawal_rule_review",
            post(super::withdrawal_rule_review::handle),
        )
        .route(
            "/api/v1/community/commands/withdrawal_preview",
            post(super::withdrawal_preview::handle),
        )
        .route(
            "/api/v1/community/commands/withdrawal_apply",
            post(super::withdrawal_apply::handle),
        )
        .route(
            "/api/v1/community/commands/withdrawal_cancel",
            post(super::withdrawal_cancel::handle),
        )
        .route(
            "/api/v1/community/commands/withdrawal_receipt_reveal",
            post(super::withdrawal_receipt_reveal::handle),
        )
}
