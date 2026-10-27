use crate::community::domain::{Code, Failure, Result, State, invalid};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Clone)]
pub struct Repository {
    pub pool: PgPool,
}
pub(super) fn database(_: sqlx::Error) -> Failure {
    Failure(
        "SERVICE_UNAVAILABLE",
        "服务暂不可用，请凭追踪编号查询后重试",
    )
}
impl Repository {
    pub async fn begin(&self) -> Result<Transaction<'_, Postgres>> {
        self.pool.begin().await.map_err(database)
    }
    pub async fn load(tx: &mut Transaction<'_, Postgres>) -> Result<State> {
        let v: Value =
            sqlx::query_scalar("SELECT document FROM crowd_state WHERE singleton=true FOR UPDATE")
                .fetch_one(&mut **tx)
                .await
                .map_err(database)?;
        serde_json::from_value(v).map_err(|_| Failure("SERVICE_UNAVAILABLE", "业务记录暂不可读取"))
    }
    pub async fn save(tx: &mut Transaction<'_, Postgres>, state: &State) -> Result<()> {
        let inactive: Vec<String> = state
            .codes
            .iter()
            .filter(|(_, c)| !c.active)
            .map(|(id, _)| id.clone())
            .collect();
        sqlx::query("UPDATE crowd_code_ownership SET active=false WHERE code=ANY($1)")
            .bind(&inactive)
            .execute(&mut **tx)
            .await
            .map_err(database)?;
        for (id, code) in &state.codes {
            let result=sqlx::query("INSERT INTO crowd_code_ownership(code,owner_id,product_id,task_id,active,version) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(code) DO UPDATE SET active=EXCLUDED.active WHERE crowd_code_ownership.owner_id=EXCLUDED.owner_id AND crowd_code_ownership.product_id IS NOT DISTINCT FROM EXCLUDED.product_id AND crowd_code_ownership.task_id IS NOT DISTINCT FROM EXCLUDED.task_id")
                .bind(id).bind(&code.owner).bind(&code.product).bind(&code.task).bind(code.active).bind(code.version as i64).execute(&mut **tx).await.map_err(database)?;
            if result.rows_affected() != 1 {
                return Err(Failure("CODE_COLLISION", "推广码已被其他对象使用"));
            }
        }
        sqlx::query("UPDATE crowd_state SET document=$1 WHERE singleton=true")
            .bind(json!(state))
            .execute(&mut **tx)
            .await
            .map_err(database)?;
        Ok(())
    }
    pub async fn commit(tx: Transaction<'_, Postgres>) -> Result<()> {
        tx.commit().await.map_err(database)
    }
    pub async fn entropy(tx: &mut Transaction<'_, Postgres>) -> Result<String> {
        sqlx::query_scalar("SELECT encode(gen_random_bytes(24),'hex')")
            .fetch_one(&mut **tx)
            .await
            .map_err(database)
    }
    pub async fn epoch(tx: &mut Transaction<'_, Postgres>) -> Result<i64> {
        sqlx::query_scalar("SELECT floor(extract(epoch FROM clock_timestamp()))::bigint")
            .fetch_one(&mut **tx)
            .await
            .map_err(database)
    }
    pub async fn public_product_image(
        tx: &mut Transaction<'_, Postgres>,
        id: &str,
    ) -> Result<(String, Vec<u8>)> {
        sqlx::query_as(
            "SELECT mime,content FROM crowd_files WHERE id=$1 AND purpose='product_image'",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(database)?
        .ok_or(Failure("NOT_FOUND", "图片不存在"))
    }
    pub async fn time(tx: &mut Transaction<'_, Postgres>) -> Result<String> {
        sqlx::query_scalar("SELECT to_char(clock_timestamp() AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"')").fetch_one(&mut **tx).await.map_err(database)
    }
    pub async fn authenticate(tx: &mut Transaction<'_, Postgres>, token: &str) -> Result<String> {
        if token.len() != 48 {
            return Err(Failure("SESSION_EXPIRED", "请重新登录"));
        }
        sqlx::query_scalar("SELECT user_id FROM crowd_sessions WHERE token_hash=encode(digest($1,'sha256'),'hex') AND expires_at>now()")
            .bind(token).fetch_optional(&mut **tx).await.map_err(database)?.ok_or(Failure("SESSION_EXPIRED","登录已失效，请重新登录"))
    }
    pub async fn login(&self, platform: &str, input: &Value, request: &str) -> Result<Value> {
        let mut tx = self.begin().await?;
        let mut state = Self::load(&mut tx).await?;
        let now = Self::time(&mut tx).await?;
        let existing: Option<String> =
            sqlx::query_scalar("SELECT user_id FROM crowd_identities WHERE platform_id=$1")
                .bind(platform)
                .fetch_optional(&mut *tx)
                .await
                .map_err(database)?;
        let id = if let Some(id) = existing {
            id
        } else {
            let bootstrap = std::env::var("CROWD_BOOTSTRAP_PLATFORM_ID")
                .is_ok_and(|v| v == platform)
                && state.users.is_empty();
            let agreement = state
                .rules
                .iter()
                .find(|r| r["kind"] == "agreements" && r["status"] == "published");
            if !bootstrap
                && (input["agreed"] != true
                    || agreement.is_none()
                    || input["agreement_id"] != agreement.unwrap()["id"])
            {
                return Err(Failure(
                    "AGREEMENT_REQUIRED",
                    "请阅读并主动同意当前服务协议与隐私说明",
                ));
            }
            let id = Self::entropy(&mut tx).await?;

            let source = input["source"]
                .as_str()
                .filter(|c| state.resolve(c).is_ok())
                .map(str::to_owned);
            state.register_user(&id,json!({"version":input["agreement_id"],"time":now,"agreed":input["agreed"],"bootstrap":bootstrap}),source);
            sqlx::query("INSERT INTO crowd_code_jobs(user_id,status) VALUES($1,'pending')")
                .bind(&id)
                .execute(&mut *tx)
                .await
                .map_err(database)?;
            if bootstrap {
                let u = state.users.get_mut(&id).unwrap();
                u.roles = vec![
                    "user".into(),
                    "rule_manager".into(),
                    "identity_reviewer".into(),
                    "promotion_manager".into(),
                    "auditor".into(),
                ];
                u.scope = vec!["*".into()];
            }
            sqlx::query("INSERT INTO crowd_identities VALUES($1,$2)")
                .bind(platform)
                .bind(&id)
                .execute(&mut *tx)
                .await
                .map_err(database)?;
            state.log(&id, "REQ-02-001+REQ-04-001", &id, request, &now);
            id
        };
        state.user(&id)?;
        if let Some(touch) = input["touch_token"].as_str() {
            let epoch = Self::epoch(&mut tx).await?;
            state.bind_touch(&id, touch, epoch)?;
        }
        let token = Self::entropy(&mut tx).await?;
        let ttl = std::env::var("CROWD_SESSION_SECONDS")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(7200);
        sqlx::query("INSERT INTO crowd_sessions VALUES(encode(digest($1,'sha256'),'hex'),$2,now()+make_interval(secs => $3))").bind(&token).bind(&id).bind(ttl as f64).execute(&mut *tx).await.map_err(database)?;
        state.log(&id, "REQ-02-002", &id, request, &now);
        Self::save(&mut tx, &state).await?;
        Self::commit(tx).await?;
        let promotion_status = if self.process_code_job(&id).await.is_ok() {
            "ready"
        } else {
            "pending"
        };
        Ok(json!({"token":token,"user_id":id,"expires_in":ttl,"promotion_status":promotion_status}))
    }
    pub async fn process_code_job(&self, user_id: &str) -> Result<()> {
        let mut tx = self.begin().await?;
        let mut state = Self::load(&mut tx).await?;
        state.user(user_id)?;
        if !state
            .codes
            .values()
            .any(|c| c.owner == user_id && c.active && c.product.is_none())
        {
            let mut candidate = Self::entropy(&mut tx).await?;
            while state.codes.contains_key(&candidate) {
                candidate = Self::entropy(&mut tx).await?;
            }
            state.codes.insert(
                candidate.clone(),
                Code {
                    owner: user_id.into(),
                    version: 1,
                    active: true,
                    product: None,
                    task: None,
                },
            );
            let now = Self::time(&mut tx).await?;
            state.log(
                user_id,
                "REQ-04-001",
                &candidate,
                "code-generation-job",
                &now,
            );
        }
        sqlx::query(
            "UPDATE crowd_code_jobs SET status='completed',attempts=attempts+1 WHERE user_id=$1",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(database)?;
        Self::save(&mut tx, &state).await?;
        Self::commit(tx).await
    }
    pub async fn retry_code_jobs(&self) -> Result<()> {
        let ids:Vec<String>=sqlx::query_scalar("SELECT user_id FROM crowd_code_jobs WHERE status='pending' ORDER BY created_at LIMIT 32").fetch_all(&self.pool).await.map_err(database)?;
        for id in ids {
            if self.process_code_job(&id).await.is_err() {
                tracing::warn!("promotion code generation remains pending");
            }
        }
        Ok(())
    }
    pub async fn web_ticket(tx: &mut Transaction<'_, Postgres>, actor: &str) -> Result<Value> {
        let ticket = Self::entropy(tx).await?;
        sqlx::query("INSERT INTO crowd_web_tickets VALUES(encode(digest($1,'sha256'),'hex'),$2,now()+interval '60 seconds')").bind(&ticket).bind(actor).execute(&mut **tx).await.map_err(database)?;
        Ok(json!({"ticket":ticket,"expires_in":60}))
    }
    pub async fn redeem(&self, ticket: &str) -> Result<Value> {
        let mut tx = self.begin().await?;
        let actor:String=sqlx::query_scalar("DELETE FROM crowd_web_tickets WHERE ticket_hash=encode(digest($1,'sha256'),'hex') AND expires_at>now() RETURNING user_id").bind(ticket).fetch_optional(&mut *tx).await.map_err(database)?.ok_or(Failure("SESSION_EXPIRED","登录票据已失效"))?;
        let state = Self::load(&mut tx).await?;
        state.user(&actor)?;
        let token = Self::entropy(&mut tx).await?;
        sqlx::query("INSERT INTO crowd_sessions VALUES(encode(digest($1,'sha256'),'hex'),$2,now()+interval '2 hours')").bind(&token).bind(&actor).execute(&mut *tx).await.map_err(database)?;
        Self::commit(tx).await?;
        Ok(json!({"token":token,"user_id":actor,"expires_in":7200}))
    }
    pub async fn logout(tx: &mut Transaction<'_, Postgres>, token: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM crowd_sessions WHERE token_hash=encode(digest($1,'sha256'),'hex')",
        )
        .bind(token)
        .execute(&mut **tx)
        .await
        .map_err(database)?;
        Ok(())
    }
    pub async fn validate_file(
        tx: &mut Transaction<'_, Postgres>,
        id: &str,
        actor: &str,
        purpose: &str,
    ) -> Result<()> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM crowd_files WHERE id=$1 AND owner_id=$2 AND purpose=$3)",
        )
        .bind(id)
        .bind(actor)
        .bind(purpose)
        .fetch_one(&mut **tx)
        .await
        .map_err(database)?;
        if exists {
            Ok(())
        } else {
            Err(Failure("FORBIDDEN", "文件不存在、用途错误或不属于本人"))
        }
    }
    pub async fn upload(
        tx: &mut Transaction<'_, Postgres>,
        actor: &str,
        purpose: &str,
        bytes: Vec<u8>,
    ) -> Result<Value> {
        if !["avatar", "wechat_receipt", "product_image"].contains(&purpose) {
            return Err(invalid("文件用途无效"));
        }
        // Engineering resource ceiling, not a confirmed business image policy.
        if bytes.len() > 5 * 1024 * 1024 {
            return Err(invalid("图片超过当前服务处理上限"));
        }
        let format =
            image::guess_format(&bytes).map_err(|_| invalid("仅支持真实 PNG/JPEG 图片"))?;
        let mime = match format {
            image::ImageFormat::Png => "image/png",
            image::ImageFormat::Jpeg => "image/jpeg",
            _ => return Err(invalid("仅支持 PNG/JPEG 图片")),
        };
        let mut reader = image::ImageReader::with_format(std::io::Cursor::new(&bytes), format);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(4096);
        limits.max_image_height = Some(4096);
        limits.max_alloc = Some(64 * 1024 * 1024);
        reader.limits(limits);
        reader
            .decode()
            .map_err(|_| invalid("图片损坏或解码资源超限"))?;
        let id:String=sqlx::query_scalar("INSERT INTO crowd_files(owner_id,purpose,mime,content) VALUES($1,$2,$3,$4) RETURNING id").bind(actor).bind(purpose).bind(mime).bind(bytes).fetch_one(&mut **tx).await.map_err(database)?;
        Ok(json!({"id":id,"purpose":purpose,"official_verified":false}))
    }
    pub async fn file(
        tx: &mut Transaction<'_, Postgres>,
        id: &str,
        actor: &str,
    ) -> Result<(String, Vec<u8>)> {
        sqlx::query_as("SELECT mime,content FROM crowd_files WHERE id=$1 AND owner_id=$2")
            .bind(id)
            .bind(actor)
            .fetch_optional(&mut **tx)
            .await
            .map_err(database)?
            .ok_or(Failure("FORBIDDEN", "无权查看文件"))
    }
}
