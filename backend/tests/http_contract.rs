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
        serde_json::json!({"data":{"type":"systems","id":"crowd","attributes":{"application":"crowd","version":env!("CARGO_PKG_VERSION"),"stage":"foundation","capabilities":["manual_settlement","privacy_copy","task_events_hmac","cursor_pagination"],"read_compatibility":"既有查询保持兼容；新类型操作需客户端支持，旧申请仍可查询"}}})
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
#[tokio::test]
async fn malformed_community_requests_have_safe_stable_errors() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://crowd:unused@127.0.0.1:1/crowd")
        .unwrap();
    let app = crowd_api::router(pool);
    for (path, body) in [
        ("/api/v1/community/login", "{"),
        ("/api/v1/catalog/touches", "{}"),
        ("/api/v1/catalog/touches", "{"),
        ("/api/v1/community/commands/profile", "{}"),
        ("/api/v1/community/login", "{\"openid\":\"victim\"}"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response.headers()["cache-control"], "no-store");
        let value: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(value["error"]["code"], "INVALID_INPUT");
        assert!(value["error"]["request_id"].as_str().unwrap().len() > 8);
        assert!(!value.to_string().contains("unused"));
    }
}

#[tokio::test]
async fn malformed_catalog_filter_uses_stable_envelope_without_database() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://crowd:unused@127.0.0.1:1/crowd")
        .unwrap();
    let response = crowd_api::router(pool)
        .oneshot(
            Request::builder()
                .uri("/api/v1/catalog?min_cents=1.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let value: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(value["error"]["code"], "INVALID_INPUT");
    assert!(value["error"]["request_id"].is_string());
}

#[tokio::test]
async fn entity_command_handlers_preserve_json_and_idempotency_validation() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://crowd:unused@127.0.0.1:1/crowd")
        .unwrap();
    let app = crowd_api::router(pool);
    for action in [
        "profile",
        "receipt",
        "receipt_cleanup",
        "settlement_exception",
        "code_rotate",
        "rule_save",
        "correction",
        "logout",
        "product_save",
        "category_save",
        "policy_publish",
        "price_save",
        "template_save",
        "task_start",
        "evidence_submit",
        "review_decide",
        "task_repair_review",
        "quality_inspect",
        "unknown_legacy_action",
    ] {
        for (body, key, message) in [
            ("{}", "", "缺少有效 Idempotency-Key"),
            ("{", "original-key", "请求必须是合法 JSON 且未超过处理上限"),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/api/v1/community/commands/{action}"))
                        .header("content-type", "application/json")
                        .header("idempotency-key", key)
                        .body(Body::from(body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::UNPROCESSABLE_ENTITY,
                "{action}"
            );
            assert_eq!(response.headers()["cache-control"], "no-store");
            assert_eq!(response.headers()["x-content-type-options"], "nosniff");
            let value: serde_json::Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap())
                    .unwrap();
            assert_eq!(value["error"]["code"], "INVALID_INPUT", "{action}");
            assert_eq!(value["error"]["message"], message, "{action}");
            assert!(
                value["error"]["request_id"]
                    .as_str()
                    .is_some_and(|id| id.len() > 8)
            );
        }
    }
}

#[tokio::test]
async fn private_file_body_limit_and_query_failures_keep_error_envelopes() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://crowd:unused@127.0.0.1:1/crowd")
        .unwrap();
    let app = crowd_api::router(pool);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/community/files/wechat_receipt")
                .body(Body::from(vec![0u8; 5 * 1024 * 1024 + 1]))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let value: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(value["error"]["code"], "INVALID_INPUT");
    assert_eq!(
        value["error"]["message"],
        "图片请求超过服务处理上限或读取失败"
    );
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/community/files/private-file?reason=a&reason=b")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let value: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(value["error"]["message"], "图片查看用途参数无效");
}
