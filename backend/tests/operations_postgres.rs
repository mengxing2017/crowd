use crate::{ApplicationServices, platform::south::port::Repository};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
#[tokio::test]
#[ignore = "requires isolated local CROWD_TEST_DATABASE_URL"]
async fn operations_concurrent_quota_idempotency_ownership_and_failure_recovery() {
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
    let ids = [
        "ops-alice",
        "ops-bob",
        "ops-operator",
        "ops-reviewer",
        "ops-verifier",
        "ops-support",
    ];
    let tokens: Vec<_> = (10..16).map(|n| format!("{n:048}")).collect();
    let mut tx = repo.begin().await.unwrap();
    let mut s = Repository::load(&mut tx).await.unwrap();
    let now = Repository::epoch(&mut tx).await.unwrap();
    for (index, id) in ids.iter().enumerate() {
        s.register_user(id, json!({}), None);
        let u = s.users.get_mut(*id).unwrap();
        u.roles = vec![
            match index {
                2 => "operator",
                3 => "settlement_reviewer",
                4 => "verifier",
                5 => "support",
                _ => "user",
            }
            .into(),
        ];
        u.scope = vec![if index > 1 { "*".into() } else { (*id).into() }];
        sqlx::query("INSERT INTO crowd_sessions(token_hash,user_id,expires_at) VALUES(encode(digest($1,'sha256'),'hex'),$2,now()+interval '1 hour')").bind(&tokens[index]).bind(id).execute(&mut *tx).await.unwrap();
    }
    s.catalog.categories.insert("ops-category".into(),json!({"id":"ops-category","name":"测试分类","enabled":true,"deleted":false,"version":1,"sort":0}));
    s.catalog
        .policies
        .push(json!({"version":1,"window_seconds":100,"strategy":"last_touch"}));
    let content = json!({"name":"隔离额度并发测试","category":"ops-category","summary":"测试","details":"测试","limitations":"仅隔离数据","estimated_minutes":1,"regions":[],"qualifications":[],"images":[{"file_id":"ops-fixture-image","alt":"隔离测试"}],"tasks":[{"id":"ops-task","name":"测试任务","version":1,"completion":"测试","evidence":"测试","rule_version":"ops-rule","commission_cents":100,"currency":"CNY","rule_starts_at":now-10,"rule_ends_at":now+10000,"capacity":10,"digital_limit_cents":null,"allow_self":true,"submit_seconds":1000}]});
    s.catalog.products.insert("ops-product".into(),json!({"id":"ops-product","version":1,"draft":content,"releases":[{"id":"ops-release","status":"approved","starts_at":now-10,"ends_at":now+10000,"content":content}]}));
    Repository::save(&mut tx, &s).await.unwrap();
    Repository::commit(tx).await.unwrap();
    let config = json!({"id":"ops-campaign","version":0,"name":"并发最后一份计划额度","products":["ops-product"],"responsible":"ops-operator","enabled":true,"limit_cents":100,"reason":"隔离测试参数"});
    app.commands
        .execute(&tokens[2], "campaign_save", &config, "ops-config", "ops")
        .await
        .unwrap();
    let body = json!({"id":"ops-product","task_id":"ops-task","release_id":"ops-release","confirmed":true});
    let (a, b) = tokio::join!(
        app.commands
            .execute(&tokens[0], "task_claim", &body, "ops-claim-a", "ops"),
        app.commands
            .execute(&tokens[1], "task_claim", &body, "ops-claim-b", "ops")
    );
    assert_ne!(
        a.is_ok(),
        b.is_ok(),
        "only one final activity quota may be claimed"
    );
    let (token, key, result) = if let Ok(a) = a {
        (&tokens[0], "ops-claim-a", a)
    } else {
        (&tokens[1], "ops-claim-b", b.unwrap())
    };
    assert_eq!(
        app.commands
            .execute(token, "task_claim", &body, key, "ops-replay")
            .await
            .unwrap(),
        result
    );
    let report = app
        .queries
        .execute(&tokens[2], "campaign_report", Some("ops-campaign"), "ops")
        .await
        .unwrap();
    assert_eq!(report["claimed"], 1);
    assert_eq!(report["used_cents"], 100);
    let loser = if token == &tokens[0] {
        &tokens[1]
    } else {
        &tokens[0]
    };
    assert!(app.commands.execute(loser,"ticket_create",&json!({"category":"withdrawal","related_id":"finance-app","description":"不允许读取其他用户单据"}),"ops-foreign","ops").await.is_err());
    // SQL failure after a valid domain mutation must not commit the case or idempotency result.
    sqlx::query("CREATE FUNCTION ops_fail_save() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.document::text LIKE '%ops-fault-description%' THEN RAISE EXCEPTION 'isolated injected write failure'; END IF; RETURN NEW; END $$").execute(&pool).await.unwrap();
    sqlx::query("CREATE TRIGGER ops_fail_save BEFORE UPDATE ON crowd_state FOR EACH ROW EXECUTE FUNCTION ops_fail_save()").execute(&pool).await.unwrap();
    let ticket = json!({"category":"registration","description":"ops-fault-description"});
    assert!(
        app.commands
            .execute(token, "ticket_create", &ticket, "ops-fault-key", "ops")
            .await
            .is_err()
    );
    sqlx::query("DROP TRIGGER ops_fail_save ON crowd_state")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP FUNCTION ops_fail_save()")
        .execute(&pool)
        .await
        .unwrap();
    let created = app
        .commands
        .execute(token, "ticket_create", &ticket, "ops-fault-key", "ops")
        .await
        .unwrap();
    assert_eq!(
        app.commands
            .execute(token, "ticket_create", &ticket, "ops-fault-key", "ops")
            .await
            .unwrap(),
        created
    );
    let messages = app
        .queries
        .execute(token, "messages", None, "ops")
        .await
        .unwrap();
    assert!(
        messages
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["business_id"] == created["id"])
    );
    let changed = json!({"category":"registration","description":"changed"});
    assert_eq!(
        app.commands
            .execute(token, "ticket_create", &changed, "ops-fault-key", "ops")
            .await
            .unwrap_err()
            .0,
        "IDEMPOTENCY_CONFLICT"
    );
    // Exported HTTP routes must dispatch to the same owned domain clients.
    use tower::ServiceExt;
    let response = crate::router(pool.clone())
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/community/read/tickets")
                .header("authorization", format!("Bearer {token}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let data: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(
        data["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["id"] == created["id"])
    );

    // Reply attachments retain both ownership and internal-note visibility at download time.
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let file = app
        .uploads
        .execute(
            &tokens[5],
            "support_attachment",
            png.into_inner(),
            "ops-file",
        )
        .await
        .unwrap();
    let reply=app.commands.execute(&tokens[5],"ticket_reply",&json!({"id":created["id"],"version":1,"reason":"内部核查","body":"仅内部可见","internal":true,"attachments":[file["id"]]}),"ops-private-reply","ops").await.unwrap();
    assert_eq!(reply["version"], 2);
    let owner_tickets = app
        .queries
        .execute(token, "tickets", None, "ops")
        .await
        .unwrap();
    assert_eq!(
        owner_tickets
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == created["id"])
            .unwrap()["replies"],
        json!([])
    );
    assert!(
        app.sensitive_files
            .execute(
                token,
                file["id"].as_str().unwrap(),
                "本人不可查看内部备注",
                "ops"
            )
            .await
            .is_err()
    );
    assert!(
        app.sensitive_files
            .execute(
                &tokens[5],
                file["id"].as_str().unwrap(),
                "客服处理本工单",
                "ops"
            )
            .await
            .is_ok()
    );
    assert!(app.commands.execute(token,"ticket_reply",&json!({"id":created["id"],"version":2,"reason":"引用他人文件","body":"不得引用","internal":false,"attachments":[file["id"]]}),"ops-foreign-file","ops").await.is_err());
    pool.close().await;
}
