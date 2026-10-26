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
#[derive(Serialize)]
pub struct Attributes {
    application: &'static str,
    version: &'static str,
    stage: &'static str,
}
#[derive(Serialize)]
pub struct Resource {
    r#type: &'static str,
    id: &'static str,
    attributes: Attributes,
}
#[derive(Serialize)]
pub struct Document {
    data: Resource,
}
pub async fn system_info(State(app): State<Application>) -> Json<Document> {
    let info = app.system.info();
    Json(Document {
        data: Resource {
            r#type: "systems",
            id: "crowd",
            attributes: Attributes {
                application: info.application,
                version: info.version,
                stage: info.stage,
            },
        },
    })
}
