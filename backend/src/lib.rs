pub mod community;
pub mod runtime;
pub mod system;
use axum::{Router, routing::get};
use sqlx::PgPool;
use system::{
    north::{local::SystemService, remote},
    south::adapter::postgres::PostgresReadiness,
};
#[derive(Clone)]
pub struct Application {
    pub system: SystemService<PostgresReadiness>,
    pub community: community::north::local::Service,
}
pub fn router(pool: PgPool) -> Router {
    Router::new()
        .merge(community::north::remote::routes())
        .route("/healthz", get(remote::liveness))
        .route("/readyz", get(remote::readiness))
        .route("/api/v1/system", get(remote::system_info))
        .with_state(Application {
            system: SystemService::new(PostgresReadiness::new(pool.clone())),
            community: community::north::local::Service {
                repo: community::south::adapter::postgres::Repository { pool },
            },
        })
}
