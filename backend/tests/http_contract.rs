use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
#[tokio::test]
async fn system_contract_and_liveness_do_not_require_database() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://localhost/crowd")
        .unwrap();
    let app = crowd_api::router(pool);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/system")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"data":{"type":"systems","id":"crowd","attributes":{"application":"crowd","version":env!("CARGO_PKG_VERSION"),"stage":"foundation"}}})
    );
    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn readiness_fails_when_database_is_unavailable() {
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(100))
        .connect_lazy("postgres://crowd:unused@127.0.0.1:1/crowd")
        .unwrap();
    let response = crowd_api::router(pool)
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value, serde_json::json!({"status":"unavailable"}));
}
