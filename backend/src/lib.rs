#![doc = include_str!("../public-surface.md")]
mod application;
pub mod audit;
pub mod commissions;
pub mod files;
pub mod governance;
pub mod identity;
pub mod platform;
pub mod promotion;
pub mod receipts;
pub mod settlements;
pub mod shared;
pub mod support;
pub mod withdrawals;
pub(crate) use application::ApplicationServices;
pub mod catalog;
#[cfg(test)]
pub(crate) mod community;
mod http;
pub mod runtime;
pub mod system;
pub mod tasks;
use axum::{Router, routing::get};
use sqlx::PgPool;
use system::{
    remote,
    south::port::{PostgresReadiness, services::SystemService},
};
#[derive(Clone)]
pub struct Application {
    pub(crate) services: ApplicationServices,
    pub(crate) system: SystemService<PostgresReadiness>,
}
pub fn router(pool: PgPool) -> Router {
    Router::new()
        .merge(platform::remote::routes())
        .route("/healthz", get(remote::liveness))
        .route("/readyz", get(remote::readiness))
        .route("/api/v1/system", get(remote::system_info))
        .with_state(Application {
            services: ApplicationServices::new(platform::south::port::Repository {
                pool: pool.clone(),
            }),
            system: SystemService::new(PostgresReadiness::new(pool.clone())),
        })
}

#[cfg(test)]
extern crate self as crowd_api;
#[cfg(test)]
#[path = "../tests/mod.rs"]
mod validation_tests;

pub mod campaigns;
pub mod notifications;
pub mod risk;
