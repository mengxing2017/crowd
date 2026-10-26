use crowd_api::community::{
    north::local::Service,
    south::{adapter::postgres::Repository, port::IdentityProvider},
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
struct TrustedFixture;
impl IdentityProvider for TrustedFixture {
    async fn exchange(&self, code: &str) -> Result<String, ()> {
        if ["alice", "bob"].contains(&code) {
            Ok(format!("test-app:{code}"))
        } else {
            Err(())
        }
    }
}
async fn login(service: &Service, code: &str) -> Value {
    service
        .login(
            &TrustedFixture,
            &json!({"code":code,"agreed":true,"agreement_id":"test-agreement","source":"invalid"}),
            "login-test",
        )
        .await
        .unwrap()
}
#[tokio::test]
#[ignore = "requires explicit isolated CROWD_TEST_DATABASE_URL; never uses CROWD_DATABASE_URL"]
async fn postgres_registration_sessions_files_and_concurrent_commands() {
    let url =
        std::env::var("CROWD_TEST_DATABASE_URL").expect("explicit isolated test database required");
    assert!(
        url.starts_with("postgres://")
            && url.contains("@127.0.0.1:")
            && url.rsplit('/').next().unwrap().starts_with("crowd_test_"),
        "test database must be a disposable local crowd_test_* database"
    );
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let service = Service {
        repo: Repository { pool: pool.clone() },
    };
    let mut tx = service.repo.begin().await.unwrap();
    let mut state = Repository::load(&mut tx).await.unwrap();
    assert!(
        state.users.is_empty(),
        "test requires an empty disposable database"
    );
    state.rules.push(json!({"id":"test-agreement","kind":"agreements","version":1,"status":"published","content":{"service":"测试协议","privacy":"测试隐私说明"}}));
    Repository::save(&mut tx, &state).await.unwrap();
    Repository::commit(tx).await.unwrap();
    assert!(
        service
            .login(
                &TrustedFixture,
                &json!({"code":"forged","openid":"alice","agreed":true}),
                "forged"
            )
            .await
            .is_err()
    );
    assert!(
        service
            .login(
                &TrustedFixture,
                &json!({"code":"alice","agreed":false}),
                "disagreed"
            )
            .await
            .is_err()
    );
    let mut tx = service.repo.begin().await.unwrap();
    let mut queued = Repository::load(&mut tx).await.unwrap();
    queued.register_user("queued", json!({"version":"test-agreement"}), None);
    sqlx::query("INSERT INTO crowd_code_jobs(user_id,status) VALUES('queued','pending')")
        .execute(&mut *tx)
        .await
        .unwrap();
    Repository::save(&mut tx, &queued).await.unwrap();
    Repository::commit(tx).await.unwrap();
    service.repo.retry_code_jobs().await.unwrap();
    service.repo.retry_code_jobs().await.unwrap();
    let mut tx = service.repo.begin().await.unwrap();
    let recovered = Repository::load(&mut tx).await.unwrap();
    assert_eq!(
        recovered
            .codes
            .values()
            .filter(|c| c.owner == "queued" && c.active)
            .count(),
        1
    );
    drop(tx);
    let (a, b) = tokio::join!(login(&service, "alice"), login(&service, "alice"));
    assert_eq!(a["user_id"], b["user_id"]);
    let alice = a["token"].as_str().unwrap();
    let uid = a["user_id"].as_str().unwrap();
    let b = login(&service, "bob").await;
    let bob = b["token"].as_str().unwrap();
    let me = service.read(alice, "me", None, "read").await.unwrap();
    assert!(me["source"].is_null());
    let codes = service.read(alice, "codes", None, "codes").await.unwrap();
    assert_eq!(codes.as_array().unwrap().len(), 1);
    let input = json!({"nickname":"Concurrent","version":1});
    let (one, two) = tokio::join!(
        service.command(alice, "profile", &input, "same-key", "one"),
        service.command(alice, "profile", &input, "same-key", "two")
    );
    assert_eq!(one.unwrap(), two.unwrap());
    assert_eq!(
        service
            .command(
                alice,
                "profile",
                &json!({"nickname":"changed","version":1}),
                "same-key",
                "changed"
            )
            .await
            .unwrap_err()
            .0,
        "IDEMPOTENCY_CONFLICT"
    );
    let recovered = service
        .read(alice, "command_result", Some("same-key"), "query-result")
        .await
        .unwrap();
    assert_eq!(recovered["version"], 2);
    let input1 = json!({"nickname":"Winner 1","version":2});
    let input2 = json!({"nickname":"Winner 2","version":2});
    let (one, two) = tokio::join!(
        service.command(alice, "profile", &input1, "race-1", "race-1"),
        service.command(alice, "profile", &input2, "race-2", "race-2")
    );
    assert_ne!(one.is_ok(), two.is_ok());
    let version = service.read(alice, "me", None, "read").await.unwrap()["version"]
        .as_u64()
        .unwrap();
    assert!(
        service
            .upload(alice, "wechat_receipt", b"fake.png".to_vec(), "fake-file")
            .await
            .is_err()
    );
    let image = image::RgbImage::from_pixel(2, 2, image::Rgb([0, 0, 0]));
    let mut bytes = std::io::Cursor::new(vec![]);
    image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    let avatar = service
        .upload(alice, "avatar", bytes.get_ref().clone(), "avatar")
        .await
        .unwrap();
    assert!(service.command(alice,"receipt",&json!({"method":"wechat","file_id":avatar["id"],"confirmed":true,"version":version}),"wrong-purpose","wrong-purpose").await.is_err());
    let receipt = service
        .upload(alice, "wechat_receipt", bytes.into_inner(), "receipt")
        .await
        .unwrap();
    let file_id = receipt["id"].as_str().unwrap();
    assert!(service.file(bob, file_id, "cross-owner").await.is_err());
    assert!(service.file(alice, file_id, "self").await.is_ok());
    {
        use axum::{
            body::{Body, to_bytes},
            http::{Request, StatusCode},
        };
        use tower::ServiceExt;
        let http = crowd_api::router(pool.clone());
        let response = http
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/community/read/me")
                    .header("authorization", format!("Bearer {alice}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(value["data"]["id"], uid);
        let response = http
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/community/files/{file_id}"))
                    .header("authorization", format!("Bearer {bob}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let own_code = codes[0]["id"].as_str().unwrap();
        let response = http
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/community/resolve/{own_code}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 16384).await.unwrap();
        assert!(!std::str::from_utf8(&body).unwrap().contains(uid));
        if std::env::var("CROWD_PROMOTION_ORIGIN").is_ok() {
            let response = http
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/v1/community/qr/{own_code}"))
                        .header("authorization", format!("Bearer {alice}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()["content-type"], "image/png");
            let image = image::load_from_memory(
                &to_bytes(response.into_body(), 1024 * 1024).await.unwrap(),
            )
            .unwrap()
            .to_luma8();
            assert!(image.width() > 200);
            assert!(image.pixels().any(|p| p[0] == 0));
            assert!(image.pixels().any(|p| p[0] == 255));
        }
    }
    let saved = service
        .command(
            alice,
            "receipt",
            &json!({"method":"wechat","file_id":file_id,"confirmed":true,"version":version}),
            "receipt-save",
            "receipt-save",
        )
        .await
        .unwrap();
    assert!(
        service
            .command(
                bob,
                "receipt_reveal",
                &json!({"id":saved["id"],"reason":"forged ownership"}),
                "reveal-other",
                "reveal-other"
            )
            .await
            .is_err()
    );
    let v = version + 1;
    service.command(alice,"receipt",&json!({"method":"alipay","account":"private@example.test","confirmed":true,"version":v}),"account","account").await.unwrap();
    let me = service.read(alice, "me", None, "masked").await.unwrap();
    assert!(!me.to_string().contains("private@example.test"));
    assert_eq!(me["receipts"][0]["file_id"], file_id);
    assert!(
        service
            .read(bob, "receipts", Some(uid), "denied")
            .await
            .is_err()
    );
    assert!(
        service
            .command(
                alice,
                "rule_save",
                &json!({"kind":"operator","version":0,"content":{}}),
                "forbidden",
                "forbidden"
            )
            .await
            .is_err()
    );
    let ticket = service
        .command(alice, "web_ticket", &json!({}), "ticket", "ticket")
        .await
        .unwrap();
    let ticket = ticket["ticket"].as_str().unwrap();
    let web = service.repo.redeem(ticket).await.unwrap();
    assert!(service.repo.redeem(ticket).await.is_err());
    service
        .command(alice, "logout", &json!({}), "logout", "logout")
        .await
        .unwrap();
    assert!(service.read(alice, "me", None, "logged-out").await.is_err());
    let web_token = web["token"].as_str().unwrap();
    assert!(service.read(web_token, "me", None, "web").await.is_ok());
    sqlx::query("UPDATE crowd_sessions SET expires_at=now()-interval '1 second'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        service
            .read(web_token, "me", None, "expired")
            .await
            .is_err()
    );
    pool.close().await;
}
