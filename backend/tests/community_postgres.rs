#[path = "support/legacy_service.rs"]
mod legacy_service;
use super::task_support;
use crowd_api::platform::south::port::{IdentityProvider, Repository};
use legacy_service::Service;
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
        // Fixed entity handlers must preserve the original service operation,
        // authenticated owner, target scope and authorization result.
        for section in [
            "me",
            "codes",
            "receipts",
            "my_claims",
            "my_attribution",
            "catalog_notifications",
            "my_tasks",
            "catalog_admin",
            "attribution_admin",
            "task_queue",
            "task_rules",
            "audit",
        ] {
            let expected = service.read(alice, section, None, "remote-parity").await;
            let response = http
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/v1/community/read/{section}"))
                        .header("authorization", format!("Bearer {alice}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let mut actual: Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap())
                    .unwrap();
            match expected {
                Ok(mut expected) => {
                    assert_eq!(status, StatusCode::OK, "{section}");
                    if let Some(object) = expected.as_object_mut() {
                        object.remove("server_time");
                    }
                    if let Some(object) = actual["data"].as_object_mut() {
                        object.remove("server_time");
                    }
                    assert_eq!(actual["data"], expected, "{section}");
                }
                Err(error) => {
                    assert!(!status.is_success(), "{section}");
                    assert_eq!(actual["error"]["code"], error.0, "{section}");
                }
            }
        }
        for _ in 0..2 {
            let response = http
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/community/commands/profile")
                        .header("authorization", format!("Bearer {alice}"))
                        .header("content-type", "application/json")
                        .header("idempotency-key", "same-key")
                        .body(Body::from(
                            json!({"nickname":"Concurrent","version":1}).to_string(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let value: Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 16384).await.unwrap())
                    .unwrap();
            assert_eq!(value["data"]["version"], 2);
        }
        assert_eq!(
            service
                .read(alice, "me", None, "replay-version")
                .await
                .unwrap()["version"],
            version
        );
        for action in [
            "product_save",
            "category_save",
            "policy_publish",
            "price_save",
            "review_claim",
        ] {
            let response = http
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/api/v1/community/commands/{action}"))
                        .header("authorization", format!("Bearer {alice}"))
                        .header("content-type", "application/json")
                        .header("idempotency-key", format!("remote-forbidden-{action}"))
                        .body(Body::from("{}"))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{action}");
        }
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
    catalog_flow(
        &service,
        &pool,
        web_token,
        bob,
        uid,
        b["user_id"].as_str().unwrap(),
    )
    .await
    .unwrap();
    tasks_flow(
        &service,
        web_token,
        bob,
        uid,
        b["user_id"].as_str().unwrap(),
    )
    .await;
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
    // Password-based staff identities do not create fake WeChat registrations.
    let secret = "admin-test-secret-2026";
    assert!(service.repo.bootstrap_staff("admin", secret).await.unwrap());
    assert!(
        !service
            .repo
            .bootstrap_staff("admin", "different-password")
            .await
            .unwrap()
    );
    let hash: String =
        sqlx::query_scalar("SELECT password_hash FROM crowd_staff_accounts WHERE username='admin'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(hash.starts_with("$argon2id$"));
    assert!(!hash.contains(secret));
    for username in ["admin", "missing"] {
        assert_eq!(
            service
                .staff_login(
                    &json!({"username":username,"password":"wrong"}),
                    "bad-login"
                )
                .await
                .unwrap_err()
                .0,
            "AUTH_INVALID"
        );
    }
    let staff = service
        .staff_login(
            &json!({"username":"admin","password":secret}),
            "staff-login",
        )
        .await
        .unwrap();
    let token = staff["token"].as_str().unwrap();
    let staff_id = staff["user_id"].as_str().unwrap();
    assert!(
        service
            .read(token, "rules", None, "staff-rules")
            .await
            .is_ok()
    );
    service
        .command(token, "logout", &json!({}), "staff-logout", "staff-logout")
        .await
        .unwrap();
    assert!(service.read(token, "me", None, "staff-out").await.is_err());
    for _ in 0..5 {
        assert!(
            service
                .staff_login(&json!({"username":"admin","password":"wrong"}), "attempt")
                .await
                .is_err()
        );
    }
    assert!(
        service
            .staff_login(&json!({"username":"admin","password":secret}), "blocked")
            .await
            .is_err()
    );
    sqlx::query("UPDATE crowd_staff_accounts SET blocked_until=now()-interval '1 second'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        service
            .staff_login(&json!({"username":"admin","password":secret}), "unblocked")
            .await
            .is_ok()
    );
    let mut tx = service.repo.begin().await.unwrap();
    let mut state = Repository::load(&mut tx).await.unwrap();
    state.users.get_mut(staff_id).unwrap().status = "disabled".into();
    Repository::save(&mut tx, &state).await.unwrap();
    Repository::commit(tx).await.unwrap();
    assert!(
        service
            .staff_login(&json!({"username":"admin","password":secret}), "disabled")
            .await
            .is_err()
    );
    assert!(!serde_json::to_string(&state).unwrap().contains(secret));
    pool.close().await;
}

async fn catalog_flow(
    service: &Service,
    pool: &sqlx::PgPool,
    alice: &str,
    bob: &str,
    alice_id: &str,
    bob_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = service.repo.begin().await.unwrap();
    let mut state = Repository::load(&mut tx).await.unwrap();
    state
        .users
        .get_mut(alice_id)
        .unwrap()
        .roles
        .extend(["operator".into(), "rule_manager".into()]);
    state.users.get_mut(alice_id).unwrap().scope = vec!["*".into()];
    state
        .users
        .get_mut(bob_id)
        .unwrap()
        .roles
        .extend(["product_reviewer".into(), "attribution_reviewer".into()]);
    state.users.get_mut(bob_id).unwrap().scope = vec!["*".into()];
    let now = Repository::epoch(&mut tx).await.unwrap();
    Repository::save(&mut tx, &state).await.unwrap();
    Repository::commit(tx).await.unwrap();
    let category = service
        .command(
            alice,
            "category_save",
            &json!({"name":"测试分类","version":0,"enabled":true,"sort":1,"reason":"测试"}),
            "catalog-category",
            "category",
        )
        .await
        .unwrap();
    service
        .command(
            alice,
            "policy_publish",
            &json!({"version":0,"window_seconds":3600,"strategy":"last_touch"}),
            "catalog-policy",
            "policy",
        )
        .await
        .unwrap();
    let image = image::RgbImage::from_pixel(2, 2, image::Rgb([0, 0, 0]));
    let mut bytes = std::io::Cursor::new(vec![]);
    image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    let image = service
        .upload(alice, "product_image", bytes.into_inner(), "product-image")
        .await
        .unwrap();
    let content = json!({"name":"真实数据库产品","category":category["id"],"summary":"公开简介","details":"产品详情","limitations":"必须完成指定任务","estimated_minutes":10,"regions":[],"qualifications":[],"images":[{"file_id":image["id"],"alt":"说明"}],"tasks":[{"id":"test-task","name":"任务","version":1,"completion":"人工核查","evidence":"任务证据","rule_version":"test-rule","commission_cents":123,"currency":"CNY","rule_starts_at":now-60,"rule_ends_at":now+7200,"allow_self":false,"submit_seconds":3600,"capacity":1,"digital_limit_cents":123}]});
    let private_file = service
        .read(alice, "me", None, "receipt-source")
        .await
        .unwrap()["receipts"][0]["file_id"]
        .clone();
    let mut invalid_content = content.clone();
    invalid_content["images"][0]["file_id"] = private_file;
    assert!(
        service
            .command(
                alice,
                "product_save",
                &json!({"content":invalid_content,"version":0}),
                "catalog-private-image",
                "private-image"
            )
            .await
            .is_err(),
        "receipt must never become a product image"
    );
    let product = service
        .command(
            alice,
            "product_save",
            &json!({"content":content,"version":0}),
            "catalog-draft",
            "draft",
        )
        .await
        .unwrap();
    let id = product["id"].as_str().unwrap();
    assert!(
        service
            .product_image(image["id"].as_str().unwrap())
            .await
            .is_err(),
        "draft images must remain private"
    );
    assert!(
        service.public_catalog(&json!({})).await.unwrap()["products"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let submitted = service
        .command(
            alice,
            "product_submit",
            &json!({"id":id,"version":1,"starts_at":now-1,"ends_at":now+1800}),
            "catalog-submit",
            "submit",
        )
        .await
        .unwrap();
    service.command(bob,"product_review",&json!({"id":id,"version":2,"release_id":submitted["release_id"],"approved":true,"reason":"内容和规则已检查"}),"catalog-review","review").await.unwrap();
    let public = service
        .public_catalog(&json!({"keyword":"数据库","min_cents":100,"max_cents":200}))
        .await
        .unwrap();
    assert_eq!(public["products"].as_array().unwrap().len(), 1);
    assert!(
        service
            .product_image(image["id"].as_str().unwrap())
            .await
            .is_ok()
    );
    {
        use axum::{
            body::{Body, to_bytes},
            http::{Request, StatusCode},
        };
        use tower::ServiceExt;
        let response = crowd_api::router(pool.clone())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/catalog?keyword=%E6%95%B0%E6%8D%AE%E5%BA%93")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["cache-control"], "no-store");
        let body: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 32768).await.unwrap()).unwrap();
        assert_eq!(
            body["data"]["products"][0]["content"]["name"],
            "真实数据库产品"
        );
        assert!(!body.to_string().contains(alice_id));
    }
    let code = service.read(alice, "codes", None, "codes").await.unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let touch_input = json!({"code":code,"scene":"integration"});
    let (t1, t2) = tokio::join!(
        service.touch(&touch_input, "same-visit", "visit1"),
        service.touch(&touch_input, "same-visit", "visit2")
    );
    let t = t1.unwrap();
    assert_eq!(t, t2.unwrap());
    service
        .command(
            bob,
            "touch_bind",
            &json!({"touch_token":t["touch_token"]}),
            "catalog-bind",
            "bind",
        )
        .await
        .unwrap();
    let queued_token = "c".repeat(48);
    sqlx::query("INSERT INTO crowd_sessions VALUES(encode(digest($1,'sha256'),'hex'),'queued',now()+interval '1 hour')").bind(&queued_token).execute(pool).await.unwrap();
    assert!(
        service
            .command(
                &queued_token,
                "touch_bind",
                &json!({"touch_token":t["touch_token"]}),
                "steal-touch",
                "steal"
            )
            .await
            .is_err()
    );
    let input = json!({"id":id,"release_id":submitted["release_id"],"task_id":"test-task","confirmed":true});
    let (one, two) = tokio::join!(
        service.command(bob, "task_claim", &input, "catalog-claim-bob", "claim1"),
        service.command(
            &queued_token,
            "task_claim",
            &input,
            "catalog-claim-queued",
            "claim2"
        )
    );
    assert_ne!(
        one.is_ok(),
        two.is_ok(),
        "last slot may only have one winner"
    );
    let (winner, winner_key, claim) = if let Ok(c) = one {
        (bob, "catalog-claim-bob", c)
    } else {
        (&*queued_token, "catalog-claim-queued", two.unwrap())
    };
    assert_eq!(
        service
            .command(winner, "task_claim", &input, winner_key, "retry")
            .await
            .unwrap(),
        claim
    );
    assert_eq!(
        service
            .read(winner, "command_result", Some(winner_key), "recover")
            .await
            .unwrap(),
        claim
    );
    assert!(
        service
            .command(
                winner,
                "task_claim",
                &json!({"id":id,"confirmed":false}),
                winner_key,
                "changed"
            )
            .await
            .is_err()
    );
    let mut edited = content.clone();
    edited["name"] = json!("尚未审核的新名称");
    service
        .command(
            alice,
            "product_save",
            &json!({"id":id,"version":3,"content":edited}),
            "catalog-edit",
            "edit",
        )
        .await
        .unwrap();
    assert_eq!(
        service
            .read(winner, "my_claims", None, "snapshot")
            .await
            .unwrap()[0]["product"]["name"],
        "真实数据库产品"
    );
    assert_eq!(
        service.public_catalog(&json!({})).await.unwrap()["products"][0]["content"]["name"],
        "真实数据库产品"
    );
    // Offline and new claims serialize under the same aggregate lock. Any later claim is rejected.
    service
        .command(
            alice,
            "product_offline",
            &json!({"id":id,"version":4,"stock_policy":"continue_existing","reason":"测试下架"}),
            "catalog-offline",
            "offline",
        )
        .await
        .unwrap();
    assert!(
        service
            .command(
                &queued_token,
                "task_claim",
                &input,
                "after-offline",
                "offline-claim"
            )
            .await
            .is_err()
    );
    assert!(
        service.public_catalog(&json!({})).await.unwrap()["products"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        service
            .product_image(image["id"].as_str().unwrap())
            .await
            .is_err()
    );
    assert_eq!(
        service
            .read(winner, "my_claims", None, "preserved")
            .await
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        service
            .read(winner, "catalog_notifications", None, "notifications")
            .await
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["kind"] == "product_offline")
    );
    let mut tx = service.repo.begin().await.unwrap();
    let state = Repository::load(&mut tx).await.unwrap();
    assert_eq!(state.catalog.claims.len(), 1);
    assert_eq!(state.catalog.touches.len(), 1);
    assert!(state.catalog.completions.is_empty());
    assert!(state.audit.iter().any(|a| a["action"] == "task_claim"));
    Ok(())
}

async fn tasks_flow(service: &Service, alice: &str, bob: &str, alice_id: &str, bob_id: &str) {
    let mut tx = service.repo.begin().await.unwrap();
    let mut s = Repository::load(&mut tx).await.unwrap();
    let now = Repository::epoch(&mut tx).await.unwrap();
    s.users
        .get_mut(bob_id)
        .unwrap()
        .roles
        .extend(["verifier".into(), "task_reviewer".into()]);
    s.users
        .get_mut(alice_id)
        .unwrap()
        .roles
        .extend(["task_reviewer".into(), "verifier".into()]);
    task_support::seed(&mut s, alice_id, bob_id, now);
    // A separate participant makes both reviewer identities independent from the task owner.
    s.register_user("task-user", json!({"version":"test-agreement"}), None);
    let user_token = Repository::entropy(&mut tx).await.unwrap();
    sqlx::query("INSERT INTO crowd_sessions VALUES(encode(digest($1,'sha256'),'hex'),'task-user',now()+interval '1 hour')").bind(&user_token).execute(&mut *tx).await.unwrap();
    s.catalog.categories.insert(
        "task-cat".into(),
        json!({"id":"task-cat","version":1,"name":"任务分类","enabled":true,"sort":1}),
    );
    let task = json!({"id":"task","name":"完成业务操作","version":1,"completion":"按步骤提供真实说明","evidence":"按表单提交","rule_version":"price","commission_cents":101,"currency":"CNY","rule_starts_at":now-100,"rule_ends_at":now+10000,"allow_self":false,"submit_seconds":2000,"capacity":1,"digital_limit_cents":101,"template_id":"template","price_id":"price"});
    let content = json!({"name":"新版任务产品","category":"task-cat","summary":"简介","details":"操作说明","limitations":"按模板审核","estimated_minutes":5,"regions":[],"qualifications":[],"images":[{"file_id":"fixture","alt":"说明"}],"tasks":[task]});
    s.catalog.products.insert("tasks-product".into(),json!({"id":"tasks-product","version":3,"draft":content,"editor":alice_id,"archived":false,"offline":false,"releases":[{"id":"tasks-release","status":"approved","starts_at":now-10,"ends_at":now+4000,"content":content,"editor":alice_id,"submitted_by":alice_id,"reviewer":bob_id}]}));
    Repository::save(&mut tx, &s).await.unwrap();
    Repository::commit(tx).await.unwrap();
    let claim_body = json!({"id":"tasks-product","task_id":"task","release_id":"tasks-release","confirmed":true});
    let (a, b) = tokio::join!(
        service.command(
            &user_token,
            "task_claim",
            &claim_body,
            "tasks-claim",
            "test"
        ),
        service.command(
            &user_token,
            "task_claim",
            &claim_body,
            "tasks-claim",
            "test"
        )
    );
    let claim = a.unwrap();
    assert_eq!(claim, b.unwrap());
    let id = claim["id"].as_str().unwrap();
    assert!(
        service
            .command(alice, "task_claim", &claim_body, "tasks-quota", "test")
            .await
            .is_err()
    );
    let t = service
        .command(
            &user_token,
            "task_start",
            &json!({"id":id,"version":1}),
            "tasks-start",
            "test",
        )
        .await
        .unwrap();
    let t = service
        .command(
            &user_token,
            "task_step",
            &json!({"id":id,"version":t["version"],"step_id":"one"}),
            "tasks-step-one",
            "test",
        )
        .await
        .unwrap();
    let t = service
        .command(
            &user_token,
            "task_step",
            &json!({"id":id,"version":t["version"],"step_id":"two"}),
            "tasks-step-two",
            "test",
        )
        .await
        .unwrap();
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 2)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    assert!(
        service
            .upload(
                &user_token,
                &format!("task_evidence:{id}"),
                b"fake-png".to_vec(),
                "fake"
            )
            .await
            .is_err()
    );
    let attachment = service
        .upload(
            &user_token,
            &format!("task_evidence:{id}"),
            png.into_inner(),
            "upload-proof",
        )
        .await
        .unwrap();
    let file_id = attachment["id"].as_str().unwrap();
    assert!(service.file(alice, file_id, "no-owner").await.is_err());
    assert!(
        service
            .evidence_file(alice, file_id, "未提交草稿不能查看", "private-draft")
            .await
            .is_err()
    );
    let (_, bytes) = service
        .file(&user_token, file_id, "own-upload")
        .await
        .unwrap();
    let t = service
        .command(
            &user_token,
            "evidence_discard",
            &json!({"id":id,"version":t["version"]}),
            "tasks-cleanup",
            "test",
        )
        .await
        .unwrap();
    assert!(service.file(&user_token, file_id, "cleaned").await.is_err());
    let attachment = service
        .upload(
            &user_token,
            &format!("task_evidence:{id}"),
            bytes,
            "upload-proof-again",
        )
        .await
        .unwrap();
    let file_id = attachment["id"].as_str().unwrap();
    let t=service.command(&user_token,"evidence_save",&json!({"id":id,"version":t["version"],"evidence":{"fields":{"description":"业务操作真实完成","order":"CASE-01","photo":[file_id]},"occurred_at":t["started_at"]}}),"tasks-draft","test").await.unwrap();
    let p = service
        .command(
            &user_token,
            "evidence_preview",
            &json!({"id":id,"version":t["version"]}),
            "tasks-preview",
            "test",
        )
        .await
        .unwrap();
    let submit =
        json!({"id":id,"version":p["version"],"preview_id":p["preview"]["id"],"confirmed":true});
    let (a, b) = tokio::join!(
        service.command(
            &user_token,
            "evidence_submit",
            &submit,
            "tasks-submit",
            "test"
        ),
        service.command(
            &user_token,
            "evidence_submit",
            &submit,
            "tasks-submit",
            "test"
        )
    );
    let t = a.unwrap();
    assert_eq!(t, b.unwrap());
    assert_eq!(t["submissions"].as_array().unwrap().len(), 1);
    assert!(
        service
            .evidence_file(alice, file_id, "已授权独立复核", "view-proof")
            .await
            .is_ok()
    );
    let acquire = json!({"id":id,"version":t["version"]});
    let (a, b) = tokio::join!(
        service.command(alice, "review_claim", &acquire, "tasks-review-a", "test"),
        service.command(bob, "review_claim", &acquire, "tasks-review-b", "test")
    );
    assert_ne!(a.is_ok(), b.is_ok());
    let (reviewer, t) = if let Ok(t) = a {
        (alice, t)
    } else {
        (bob, b.unwrap())
    };
    assert!(t["draft"].is_null());
    assert!(t["receipt"]["evidence"].is_null());
    let decision = json!({"id":id,"version":t["version"],"decision":"approve","reason":"逐项核对完成","checklist_confirmed":true});
    let (a, b) = tokio::join!(
        service.command(
            reviewer,
            "review_decide",
            &decision,
            "tasks-approve",
            "test"
        ),
        service.command(
            reviewer,
            "review_decide",
            &decision,
            "tasks-approve",
            "test"
        )
    );
    assert_eq!(a.unwrap(), b.unwrap());
    let detail = service
        .read(&user_token, "task_detail", Some(id), "receipt-after-reopen")
        .await
        .unwrap();
    assert_eq!(detail["status"], "approved");
    let mut tx = service.repo.begin().await.unwrap();
    let state = Repository::load(&mut tx).await.unwrap();
    assert_eq!(state.tasks.confirmations.len(), 1);
    assert_eq!(state.tasks.changes.len(), 1);
    assert_eq!(
        state.tasks.confirmations[id]["calculation"]["amount_cents"],
        101
    );
    assert_eq!(state.tasks.confirmations[id]["withdrawable"], true);
    assert!(
        state
            .audit
            .iter()
            .any(|a| a["action"] == "review_decide" && a["rule_version"] == "M08-M12-v1")
    );
    drop(tx);
    let mut tx = service.repo.begin().await.unwrap();
    let mut state = Repository::load(&mut tx).await.unwrap();
    task_support::instance(&mut state, "withdraw-race", "task-user", now);
    let completed = state.tasks.instances[id].clone();
    let race = state.tasks.instances.get_mut("withdraw-race").unwrap();
    for field in ["receipt", "submissions", "steps", "draft", "submitted_at"] {
        race[field] = completed[field].clone();
    }
    race["status"] = json!("submitted");
    Repository::save(&mut tx, &state).await.unwrap();
    Repository::commit(tx).await.unwrap();
    let withdraw = json!({"id":"withdraw-race","version":1,"reason":"用户改正证据"});
    let acquire = json!({"id":"withdraw-race","version":1});
    let (withdrawn, claimed) = tokio::join!(
        service.command(
            &user_token,
            "evidence_withdraw",
            &withdraw,
            "withdraw-race",
            "test"
        ),
        service.command(alice, "review_claim", &acquire, "claim-race", "test")
    );
    assert_ne!(withdrawn.is_ok(), claimed.is_ok());
    let t = service
        .read(
            &user_token,
            "task_detail",
            Some("withdraw-race"),
            "race-result",
        )
        .await
        .unwrap();
    assert!(t["status"] == "in_progress" || t["status"] == "reviewing");
    // Fault injection: the second selected queue entry already has a confirmation after recovery.
    // The first entry must not remain approved when this later business check fails.
    let mut tx = service.repo.begin().await.unwrap();
    let mut state = Repository::load(&mut tx).await.unwrap();
    let original = state.tasks.instances[id].clone();
    for key in ["batch-a", "batch-b"] {
        let mut task = original.clone();
        task["id"] = json!(key);
        task["version"] = json!(1);
        task["status"] = json!("reviewing");
        task["reviewer"] = json!(alice_id);
        state.tasks.instances.insert(key.into(), task);
    }
    let prior_confirmation = state.tasks.confirmations[id].clone();
    state
        .tasks
        .confirmations
        .insert("batch-b".into(), prior_confirmation);
    let change_count = state.tasks.changes.len();
    Repository::save(&mut tx, &state).await.unwrap();
    Repository::commit(tx).await.unwrap();
    let entries = json!([{"id":"batch-a","version":1},{"id":"batch-b","version":1}]);
    let preview = service
        .command(
            alice,
            "review_batch_preview",
            &json!({"entries":entries}),
            "batch-fault-preview",
            "test",
        )
        .await
        .unwrap();
    assert!(service.command(alice,"review_batch",&json!({"entries":entries,"preview_id":preview["id"],"confirmed":true,"reason":"批量故障恢复测试"}),"batch-fault-commit","test").await.is_err());
    let mut tx = service.repo.begin().await.unwrap();
    let state = Repository::load(&mut tx).await.unwrap();
    assert_eq!(state.tasks.instances["batch-a"]["status"], "reviewing");
    assert!(!state.tasks.confirmations.contains_key("batch-a"));
    assert_eq!(state.tasks.changes.len(), change_count);
}
