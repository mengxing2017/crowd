//! Staff credentials are separate from trusted WeChat end-user identities.
use crate::platform::south::port::rules::{Failure, Result};
use crate::platform::south::port::{Repository, database};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use serde_json::{Value, json};
use sqlx::Row;

const INVALID: Failure = Failure("AUTH_INVALID", "账号或密码错误，或账号暂不可用");
fn unavailable() -> Failure {
    Failure("SERVICE_UNAVAILABLE", "登录服务暂不可用")
}

impl Repository {
    /// Explicit bootstrap only; existing credentials and role grants are never overwritten.
    pub async fn bootstrap_staff(&self, username: &str, password: &str) -> Result<bool> {
        if username.is_empty()
            || username.len() > 64
            || !username
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
            || !(12..=128).contains(&password.len())
        {
            return Err(Failure(
                "INVALID_INPUT",
                "管理员账号须为 1–64 位字母、数字或 _-.，密码须为 12–128 字节",
            ));
        }
        let mut tx = self.begin().await?;
        let mut state = Self::load(&mut tx).await?;
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM crowd_staff_accounts WHERE username=$1)",
        )
        .bind(username)
        .fetch_one(&mut *tx)
        .await
        .map_err(database)?;
        if exists {
            return Ok(false);
        }
        let salt = Self::entropy(&mut tx).await?;
        let password = password.to_owned();
        let hash = tokio::task::spawn_blocking(move || {
            let salt = SaltString::encode_b64(salt.as_bytes()).map_err(|_| unavailable())?;
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|_| unavailable())
        })
        .await
        .map_err(|_| unavailable())??;
        let id = Self::entropy(&mut tx).await?;
        state.register_user(&id, json!({"channel":"staff_bootstrap"}), None);
        let user = state.users.get_mut(&id).expect("created staff");
        user.nickname = "平台管理员".into();
        user.roles = [
            "rule_manager",
            "identity_reviewer",
            "promotion_manager",
            "auditor",
            "operator",
            "product_reviewer",
            "product_emergency",
            "attribution_reviewer",
            "support",
            "risk_manager",
        ]
        .map(str::to_owned)
        .to_vec();
        user.scope = vec!["*".into()];
        sqlx::query(
            "INSERT INTO crowd_staff_accounts(username,user_id,password_hash) VALUES($1,$2,$3)",
        )
        .bind(username)
        .bind(&id)
        .bind(hash)
        .execute(&mut *tx)
        .await
        .map_err(database)?;
        let now = Self::time(&mut tx).await?;
        state.log(&id, "staff_bootstrap", &id, "startup", &now);
        Self::save(&mut tx, &state).await?;
        Self::commit(tx).await?;
        Ok(true)
    }

    pub async fn staff_login(
        &self,
        username: &str,
        password: &str,
        request: &str,
    ) -> Result<Value> {
        if username.is_empty() || username.len() > 64 || password.is_empty() || password.len() > 128
        {
            return Err(INVALID);
        }
        let mut tx = self.begin().await?;
        let row = sqlx::query("SELECT user_id,password_hash,COALESCE(blocked_until>now(),false) AS blocked FROM crowd_staff_accounts WHERE username=$1 FOR UPDATE").bind(username).fetch_optional(&mut *tx).await.map_err(database)?;
        // Verify a real hash even for unknown names to avoid exposing existing usernames by timing.
        let hash: Option<String> = match &row {
            Some(row) => Some(row.get("password_hash")),
            None => sqlx::query_scalar("SELECT password_hash FROM crowd_staff_accounts LIMIT 1")
                .fetch_optional(&mut *tx)
                .await
                .map_err(database)?,
        };
        let password = password.to_owned();
        let valid = tokio::task::spawn_blocking(move || {
            hash.and_then(|hash| {
                PasswordHash::new(&hash).ok().map(|h| {
                    Argon2::default()
                        .verify_password(password.as_bytes(), &h)
                        .is_ok()
                })
            })
            .unwrap_or(false)
        })
        .await
        .map_err(|_| unavailable())?;
        let Some(row) = row else {
            let mut state = Self::load(&mut tx).await?;
            let now = Self::time(&mut tx).await?;
            state.log_failure(
                "anonymous",
                "staff_login",
                "platform",
                request,
                &now,
                "AUTH_INVALID",
            );
            Self::save(&mut tx, &state).await?;
            Self::commit(tx).await?;
            return Err(INVALID);
        };
        let id: String = row.get("user_id");
        let blocked: bool = row.get("blocked");
        let mut state = Self::load(&mut tx).await?;
        let now = Self::time(&mut tx).await?;
        let allowed = state
            .user(&id)
            .is_ok_and(|u| u.status == "active" && u.roles.iter().any(|role| role != "user"));
        if !valid || blocked || !allowed {
            if !blocked {
                sqlx::query("UPDATE crowd_staff_accounts SET failed_attempts=CASE WHEN blocked_until<=now() THEN 1 ELSE failed_attempts+1 END, blocked_until=CASE WHEN blocked_until<=now() THEN NULL WHEN failed_attempts>=4 THEN now()+interval '15 minutes' ELSE blocked_until END WHERE username=$1").bind(username).execute(&mut *tx).await.map_err(database)?;
            }
            state.log_failure(&id, "staff_login", &id, request, &now, "AUTH_INVALID");
            Self::save(&mut tx, &state).await?;
            Self::commit(tx).await?;
            return Err(INVALID);
        }
        sqlx::query("UPDATE crowd_staff_accounts SET failed_attempts=0,blocked_until=NULL WHERE username=$1").bind(username).execute(&mut *tx).await.map_err(database)?;
        let token = Self::entropy(&mut tx).await?;
        sqlx::query("INSERT INTO crowd_sessions(token_hash,user_id,expires_at) VALUES(encode(digest($1,'sha256'),'hex'),$2,now()+interval '2 hours')").bind(&token).bind(&id).execute(&mut *tx).await.map_err(database)?;
        state.log(&id, "staff_login", &id, request, &now);
        Self::save(&mut tx, &state).await?;
        Self::commit(tx).await?;
        Ok(json!({"token":token,"user_id":id,"expires_in":7200}))
    }
}
