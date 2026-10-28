mod application;
pub mod audit;
pub mod files;
pub mod governance;
pub mod identity;
pub mod platform;
pub mod promotion;
pub mod receipts;
pub mod shared;
pub mod support;
pub use application::ApplicationServices;
pub mod catalog;
pub mod community;
mod http;
pub mod runtime;
pub mod system;
pub mod tasks;
use axum::{Router, routing::get};
use sqlx::PgPool;
use system::{
    north::{local::SystemService, remote},
    south::adapter::postgres::PostgresReadiness,
};
#[derive(Clone)]
pub struct Application {
    pub services: ApplicationServices,
    pub system: SystemService<PostgresReadiness>,
    pub community: community::north::local::Service,
}
pub fn router(pool: PgPool) -> Router {
    Router::new()
        .merge(platform::north::remote::routes())
        .route("/healthz", get(remote::liveness))
        .route("/readyz", get(remote::readiness))
        .route("/api/v1/system", get(remote::system_info))
        .with_state(Application {
            services: ApplicationServices::new(platform::south::adapter::postgres::Repository {
                pool: pool.clone(),
            }),
            system: SystemService::new(PostgresReadiness::new(pool.clone())),
            community: community::north::local::Service {
                repo: platform::south::adapter::postgres::Repository { pool },
            },
        })
}
