use crate::{ApplicationServices, platform::south::port::Repository};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
#[tokio::test]
#[ignore = "requires isolated local CROWD_TEST_DATABASE_URL"]
async fn governance_transaction_audit_export_reauthentication_and_signed_events() {
    let url = std::env::var("CROWD_TEST_DATABASE_URL").expect("isolated database required");
    assert!(
        url.starts_with("postgres://")
            && url.contains("@127.0.0.1:")
            && url.rsplit('/').next().unwrap().starts_with("crowd_test_")
    );
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = Repository { pool: pool.clone() };
    let app = ApplicationServices::new(repo.clone());
    let mut tx = repo.begin().await.unwrap();
    let token_value = Repository::entropy(&mut tx).await.unwrap();
    let token = token_value.as_str();
    let mut s = Repository::load(&mut tx).await.unwrap();
    s.register_user("gov-admin", json!({}), None);
    let u = s.users.get_mut("gov-admin").unwrap();
    u.roles = vec![
        "auditor".into(),
        "operator".into(),
        "identity_reviewer".into(),
        "rule_manager".into(),
    ];
    u.scope = vec!["*".into()];
    sqlx::query("INSERT INTO crowd_sessions(token_hash,user_id,expires_at) VALUES(encode(digest($1,'sha256'),'hex'),'gov-admin',now()+interval '1 hour')").bind(token).execute(&mut *tx).await.unwrap();
    let now = Repository::epoch(&mut tx).await.unwrap();
    s.register_user("gov-alice", json!({}), None);
    s.tasks.instances.insert("gov-task".into(),json!({"id":"gov-task","owner":"gov-alice","status":"in_progress","started_at":now-100,"execute_until":now+100,"template":{"external_source":"governance_test"}}));
    Repository::save(&mut tx, &s).await.unwrap();
    Repository::commit(tx).await.unwrap();
    let i = json!({"from":0,"until":now,"reason":"隔离测试","expires_seconds":100});
    let (a, b) = tokio::join!(
        app.commands
            .execute(token, "export_request", &i, "gov-export", "gov-a"),
        app.commands
            .execute(token, "export_request", &i, "gov-export", "gov-b")
    );
    let export = a.unwrap();
    assert_eq!(export, b.unwrap());
    let id = export["id"].as_str().unwrap();
    assert!(
        app.queries
            .execute(token, "export_download", Some(id), "gov-before-ready")
            .await
            .is_err()
    );
    crate::reporting::south::port::services::ProcessExports::new(repo.clone())
        .execute()
        .await
        .unwrap();
    let download = app
        .queries
        .execute(token, "export_download", Some(id), "gov-download")
        .await
        .unwrap();
    assert_eq!(download["id"], id);
    // A database failure cannot remove an audit prefix or any historical registration.
    assert!(
        sqlx::query("UPDATE crowd_state SET document=jsonb_set(document,'{audit}','[]'::jsonb)")
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("DELETE FROM crowd_state")
            .execute(&pool)
            .await
            .is_err()
    );
    let mut tx = repo.begin().await.unwrap();
    let mut s = Repository::load(&mut tx).await.unwrap();
    s.security.policy = json!({"version":1,"reauth_seconds":60,"actions":["export_request"]});
    Repository::save(&mut tx, &s).await.unwrap();
    Repository::commit(tx).await.unwrap();
    assert_eq!(
        app.commands
            .execute(token, "export_request", &i, "gov-reauth", "gov")
            .await
            .unwrap_err()
            .0,
        "REAUTH_REQUIRED"
    );
    sqlx::query("INSERT INTO crowd_session_reauthentication(token_hash,policy_version) VALUES(encode(digest($1,'sha256'),'hex'),1)").bind(token).execute(&pool).await.unwrap();
    assert!(
        app.commands
            .execute(token, "export_request", &i, "gov-reauth", "gov")
            .await
            .is_ok()
    );
    sqlx::query("UPDATE crowd_session_reauthentication SET verified_at=now()-interval '2 minutes' WHERE token_hash=encode(digest($1,'sha256'),'hex')").bind(token).execute(&pool).await.unwrap();
    assert_eq!(
        app.commands
            .execute(token, "export_request", &i, "gov-expired", "gov")
            .await
            .unwrap_err()
            .0,
        "REAUTH_REQUIRED"
    );
    // Signed task events use independent deployment credentials; no payment processing is involved.
    let source = "governance_test";
    let body=serde_json::to_vec(&json!({"event_id":"gov-event","claim_id":"gov-task","occurred_at":now,"sent_at":now,"base_cents":30})).unwrap();
    let signature: String =
        sqlx::query_scalar("SELECT encode(hmac($1::bytea,$2::bytea,'sha256'),'hex')")
            .bind(&body)
            .bind(b"governance-test-secret-32-characters".as_slice())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        app.task_events
            .execute(source, "invalid", &body, "gov-signature")
            .await
            .is_err()
    );
    let first = app
        .task_events
        .execute(source, &signature, &body, "gov-event")
        .await
        .unwrap();
    assert_eq!(
        first,
        app.task_events
            .execute(source, &signature, &body, "gov-event-replay")
            .await
            .unwrap()
    );
    let changed=serde_json::to_vec(&json!({"event_id":"gov-event","claim_id":"gov-task","occurred_at":now,"sent_at":now,"base_cents":31})).unwrap();
    let sig: String = sqlx::query_scalar("SELECT encode(hmac($1::bytea,$2::bytea,'sha256'),'hex')")
        .bind(&changed)
        .bind(b"governance-test-secret-32-characters".as_slice())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        app.task_events
            .execute(source, &sig, &changed, "gov-conflict")
            .await
            .unwrap_err()
            .0,
        "IDEMPOTENCY_CONFLICT"
    );
    let mut tx = repo.begin().await.unwrap();
    let mut s = Repository::load(&mut tx).await.unwrap();
    assert!(
        s.tasks
            .event_receipts
            .values()
            .any(|v| v["status"] == "conflict_review")
    );
    assert!(s.audit.iter().any(|v| v["request_id"] == "gov-download"));
    s.tasks.disabled_sources.push(source.into());
    s.users.get_mut("gov-admin").unwrap().roles.clear();
    Repository::save(&mut tx, &s).await.unwrap();
    Repository::commit(tx).await.unwrap();
    assert!(
        app.queries
            .execute(token, "export_download", Some(id), "gov-revoked")
            .await
            .is_err()
    );
    assert!(
        app.task_events
            .execute(source, &signature, &body, "gov-disabled")
            .await
            .is_err()
    );
    // The old array query remains available; pagination is additive at the HTTP edge.
    use tower::ServiceExt;
    let response = crate::router(pool.clone())
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/community/read/my_claims?limit=1")
                .header("authorization", format!("Bearer {token}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let envelope: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(envelope["data"]["items"].is_array());
    pool.close().await;
}
