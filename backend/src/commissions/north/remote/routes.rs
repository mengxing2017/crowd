use crate::Application;
use axum::{
    Router,
    routing::{get, post},
};
pub fn routes() -> Router<Application> {
    Router::new()
        .route(
            "/api/v1/community/read/commission_source",
            get(super::commission_source::handle),
        )
        .route(
            "/api/v1/community/read/commission_summary",
            get(super::commission_summary::handle),
        )
        .route(
            "/api/v1/community/read/commission_entries",
            get(super::commission_entries::handle),
        )
        .route(
            "/api/v1/community/read/commission_proposals",
            get(super::commission_proposals::handle),
        )
        .route(
            "/api/v1/community/read/commission_disputes",
            get(super::commission_disputes::handle),
        )
        .route(
            "/api/v1/community/commands/commission_adjust_propose",
            post(super::commission_adjust_propose::handle),
        )
        .route(
            "/api/v1/community/commands/commission_adjust_review",
            post(super::commission_adjust_review::handle),
        )
        .route(
            "/api/v1/community/commands/commission_dispute",
            post(super::commission_dispute::handle),
        )
        .route(
            "/api/v1/community/commands/commission_dispute_resolve",
            post(super::commission_dispute_resolve::handle),
        )
}
