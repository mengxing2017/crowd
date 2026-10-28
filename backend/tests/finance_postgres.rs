use crowd_api::{
    ApplicationServices, commissions::domain as ledger,
    platform::south::adapter::postgres::Repository,
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
async fn command(app: &ApplicationServices, token: &str, a: &str, b: Value, key: &str) -> Value {
    app.commands
        .execute(token, a, &b, key, "finance-integration")
        .await
        .unwrap()
}
async fn query(app: &ApplicationServices, token: &str, section: &str) -> Value {
    app.queries
        .execute(token, section, None, "finance-integration")
        .await
        .unwrap()
}
#[tokio::test]
#[ignore = "requires isolated local CROWD_TEST_DATABASE_URL"]
async fn concurrent_holds_cancel_claim_race_idempotency_and_registration_failure_recovery() {
    let url =
        std::env::var("CROWD_TEST_DATABASE_URL").expect("explicit isolated database required");
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
    let tokens: Vec<String> = (1..=7).map(|n| n.to_string().repeat(48)).collect();
    let ids = [
        "finance-alice",
        "finance-bob",
        "finance-manager",
        "finance-reviewer",
        "finance-handler",
        "finance-verifier",
        "finance-other",
    ];
    let mut tx = repo.begin().await.unwrap();
    let mut s = Repository::load(&mut tx).await.unwrap();
    for (index, id) in ids.iter().enumerate() {
        s.register_user(id, json!({}), None);
        let u = s.users.get_mut(*id).unwrap();
        let role = match index {
            2 => "rule_manager",
            3 => "settlement_reviewer",
            4 | 6 => "manual_handler",
            5 => "verifier",
            _ => "user",
        };
        u.roles = vec![role.into()];
        u.scope = if index == 4 {
            vec!["finance-alice".into()]
        } else if index > 1 {
            vec!["*".into()]
        } else {
            vec![(*id).into()]
        };
        sqlx::query("INSERT INTO crowd_sessions(token_hash,user_id,expires_at) VALUES(encode(digest($1,'sha256'),'hex'),$2,now()+interval '1 hour')").bind(&tokens[index]).bind(id).execute(&mut *tx).await.unwrap();
    }
    ledger::append(
        &mut s,
        "finance-seed",
        "finance-alice",
        "confirmed",
        1000,
        "finance-task",
        "finance-reviewer",
        100,
        json!({"test_only":true}),
    )
    .unwrap();
    Repository::save(&mut tx, &s).await.unwrap();
    Repository::commit(tx).await.unwrap();
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(1, 1)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let receipt_file = app
        .uploads
        .execute(
            &tokens[0],
            "wechat_receipt",
            bytes.get_ref().clone(),
            "upload-receipt",
        )
        .await
        .unwrap();
    let receipt = command(
        &app,
        &tokens[0],
        "receipt",
        json!({"method":"wechat","file_id":receipt_file["id"],"confirmed":true,"version":1}),
        "finance-receipt",
    )
    .await;
    let rule=command(&app,&tokens[2],"withdrawal_rule_propose",json!({"min_cents":100,"max_cents":1000,"daily_count":20,"daily_cents":10000,"lifetime_cents":100000,"second_review_cents":0,"reminder_seconds":1,"registration_verification":true,"separate_roles":true,"methods":["wechat"],"reason":"隔离数据库测试规则，非正式门槛"}),"finance-rule").await;
    command(
        &app,
        &tokens[5],
        "withdrawal_rule_review",
        json!({"id":rule["id"],"approve":true,"reason":"隔离测试独立复核"}),
        "finance-rule-review",
    )
    .await;
    let sum = query(&app, &tokens[0], "commission_summary").await;
    let body = json!({"amount_cents":600,"receipt_id":receipt["id"],"receipt_version":receipt["version"],"summary_version":sum["version"]});
    let p1 = command(
        &app,
        &tokens[0],
        "withdrawal_preview",
        body.clone(),
        "finance-preview-1",
    )
    .await;
    let p2 = command(
        &app,
        &tokens[0],
        "withdrawal_preview",
        body.clone(),
        "finance-preview-2",
    )
    .await;
    let mut b1 = body.clone();
    b1["preview_id"] = p1["id"].clone();
    b1["confirmed"] = json!(true);
    let mut b2 = body;
    b2["preview_id"] = p2["id"].clone();
    b2["confirmed"] = json!(true);
    let (a, b) = tokio::join!(
        app.commands.execute(
            &tokens[0],
            "withdrawal_apply",
            &b1,
            "finance-apply-1",
            "race"
        ),
        app.commands.execute(
            &tokens[0],
            "withdrawal_apply",
            &b2,
            "finance-apply-2",
            "race"
        )
    );
    assert_ne!(a.is_ok(), b.is_ok());
    let (application, original, key) = if let Ok(a) = a {
        (a, b1, "finance-apply-1")
    } else {
        (b.unwrap(), b2, "finance-apply-2")
    };
    let id = application["id"].as_str().unwrap();
    let retry = command(&app, &tokens[0], "withdrawal_apply", original.clone(), key).await;
    assert_eq!(retry, application);
    let result = app
        .queries
        .execute(&tokens[0], "command_result", Some(key), "lookup-timeout")
        .await
        .unwrap();
    assert_eq!(result, application);
    assert_eq!(
        query(&app, &tokens[0], "commission_summary").await["held_cents"],
        600
    );
    let mut changed = original;
    changed["amount_cents"] = json!(100);
    assert!(
        app.commands
            .execute(&tokens[0], "withdrawal_apply", &changed, key, "conflict")
            .await
            .is_err()
    );
    assert!(
        app.queries
            .execute(&tokens[1], "withdrawal_detail", Some(id), "unauthorized")
            .await
            .is_err()
    );
    use tower::ServiceExt;
    let response = crowd_api::router(pool.clone())
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/community/commands/withdrawal_apply")
                .header("authorization", format!("Bearer {}", tokens[0]))
                .header("idempotency-key", "finance-http-stale")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(changed.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::CONFLICT);
    let raw = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let context: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(context["error"]["details"]["summary"]["held_cents"], 600);
    assert_eq!(
        context["error"]["details"]["applications"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    command(&app,&tokens[3],"settlement_review",json!({"id":id,"version":1,"approve":true,"source_checked":true,"receipt_checked":true,"reason":"核对数字来源和原资料"}),"finance-review").await;
    let cancel = json!({"id":id,"version":2,"reason":"用户撤销"});
    let claim = json!({"id":id,"version":2,"reason":"独占领取"});
    let (cancelled, claimed) = tokio::join!(
        app.commands.execute(
            &tokens[0],
            "withdrawal_cancel",
            &cancel,
            "finance-cancel",
            "race"
        ),
        app.commands.execute(
            &tokens[4],
            "settlement_claim",
            &claim,
            "finance-claim",
            "race"
        )
    );
    assert_ne!(cancelled.is_ok(), claimed.is_ok());
    // If cancellation wins, create a fresh application; otherwise continue the exclusively claimed one.
    let app_id = if cancelled.is_ok() {
        let sum = query(&app, &tokens[0], "commission_summary").await;
        let body = json!({"amount_cents":600,"receipt_id":receipt["id"],"receipt_version":receipt["version"],"summary_version":sum["version"]});
        let p = command(
            &app,
            &tokens[0],
            "withdrawal_preview",
            body.clone(),
            "finance-preview-new",
        )
        .await;
        let mut body = body;
        body["preview_id"] = p["id"].clone();
        body["confirmed"] = json!(true);
        let a = command(
            &app,
            &tokens[0],
            "withdrawal_apply",
            body,
            "finance-apply-new",
        )
        .await;
        let id = a["id"].as_str().unwrap().to_owned();
        command(&app,&tokens[3],"settlement_review",json!({"id":id,"version":1,"approve":true,"receipt_checked":true,"source_checked":true,"reason":"核对"}),"finance-review-new").await;
        command(
            &app,
            &tokens[4],
            "settlement_claim",
            json!({"id":id,"version":2,"reason":"独占领取"}),
            "finance-claim-new",
        )
        .await;
        id
    } else {
        id.to_owned()
    };
    let unknown = json!({"id":app_id,"version":3,"reason":"线下结果不明，保留占用"});
    command(
        &app,
        &tokens[4],
        "settlement_unknown",
        unknown,
        "finance-unknown",
    )
    .await;
    assert_eq!(
        query(&app, &tokens[0], "commission_summary").await["held_cents"],
        600
    );
    let voucher = app
        .uploads
        .execute(
            &tokens[4],
            "settlement_voucher",
            bytes.into_inner(),
            "voucher-upload",
        )
        .await
        .unwrap();
    let mut tx = repo.begin().await.unwrap();
    let epoch = Repository::epoch(&mut tx).await.unwrap();
    drop(tx);
    let register = json!({"id":app_id,"version":4,"reason":"核查原线下付款后登记","amount_cents":600,"receipt_id":receipt["id"],"receipt_confirmed":true,"paid_at":epoch-1,"voucher_matches":true,"voucher_file_id":voucher["id"]});
    // Abort the state update inside PostgreSQL; no registration, settlement event or idempotency result may survive.
    sqlx::query("CREATE FUNCTION finance_fail_write() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.document->'withdrawals'->'applications'->TG_ARGV[0]->'registration' IS DISTINCT FROM 'null'::jsonb THEN RAISE EXCEPTION 'isolated fault'; END IF; RETURN NEW; END $$").execute(&pool).await.unwrap();
    let trigger = format!(
        "CREATE TRIGGER finance_fail BEFORE UPDATE ON crowd_state FOR EACH ROW EXECUTE FUNCTION finance_fail_write('{}')",
        app_id
    );
    sqlx::query(&trigger).execute(&pool).await.unwrap();
    assert!(
        app.commands
            .execute(
                &tokens[4],
                "settlement_register",
                &register,
                "finance-register",
                "fault"
            )
            .await
            .is_err()
    );
    sqlx::query("DROP TRIGGER finance_fail ON crowd_state")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP FUNCTION finance_fail_write()")
        .execute(&pool)
        .await
        .unwrap();
    let a = app
        .queries
        .execute(&tokens[4], "withdrawal_detail", Some(&app_id), "recover")
        .await
        .unwrap();
    assert!(a["registration"].is_null());
    assert_eq!(a["version"], 4);
    assert!(
        app.queries
            .execute(
                &tokens[4],
                "command_result",
                Some("finance-register"),
                "recover"
            )
            .await
            .is_err()
    );
    let registered = command(
        &app,
        &tokens[4],
        "settlement_register",
        register.clone(),
        "finance-register",
    )
    .await;
    assert_eq!(registered["status"], "awaiting_verification");
    assert_eq!(
        command(
            &app,
            &tokens[4],
            "settlement_register",
            register,
            "finance-register"
        )
        .await,
        registered
    );
    assert!(
        app.sensitive_files
            .execute(
                &tokens[1],
                voucher["id"].as_str().unwrap(),
                "越权查看",
                "test"
            )
            .await
            .is_err()
    );
    assert!(
        app.sensitive_files
            .execute(
                &tokens[0],
                voucher["id"].as_str().unwrap(),
                "本人核对人工凭证",
                "test"
            )
            .await
            .is_ok()
    );
    let verified=command(&app,&tokens[5],"settlement_verify",json!({"id":app_id,"version":5,"approve":true,"voucher_checked":true,"reason":"独立核对原登记"}),"finance-verify").await;
    assert_eq!(verified["status"], "settled");
    let sum = query(&app, &tokens[0], "commission_summary").await;
    assert_eq!(sum["available_cents"], 400);
    assert_eq!(sum["held_cents"], 0);
    assert_eq!(sum["settled_cents"], 600);
    assert_eq!(sum["consistent"], true);
}
