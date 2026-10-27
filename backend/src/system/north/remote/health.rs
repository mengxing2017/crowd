use crate::Application;
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
#[derive(Serialize)]
pub struct Health {
    status: &'static str,
}
pub async fn liveness() -> Json<Health> {
    Json(Health { status: "ok" })
}
pub async fn readiness(State(app): State<Application>) -> (StatusCode, Json<Health>) {
    if app.system.ready().await {
        (StatusCode::OK, Json(Health { status: "ready" }))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(Health {
                status: "unavailable",
            }),
        )
    }
}
