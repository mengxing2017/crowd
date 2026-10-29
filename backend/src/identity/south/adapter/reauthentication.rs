use crate::platform::south::port::{Repository, State, database};
use crate::shared::{Failure, Result};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde_json::{Value, json};
use sqlx::{Postgres, Row, Transaction};
impl Repository {
    pub async fn reauthenticate(
        &self,
        token: &str,
        password: &str,
        request: &str,
    ) -> Result<Value> {
        if password.is_empty() || password.len() > 128 {
            return Err(Failure("AUTH_INVALID", "再次验证失败"));
        }
        let mut tx = self.begin().await?;
        let actor = Self::authenticate(&mut tx, token).await?;
        let row=sqlx::query("SELECT password_hash,COALESCE(blocked_until>now(),false) AS blocked FROM crowd_staff_accounts WHERE user_id=$1 FOR UPDATE").bind(&actor).fetch_optional(&mut *tx).await.map_err(database)?.ok_or(Failure("AUTH_INVALID","此身份不支持后台密码再次验证"))?;
        let hash: String = row.get("password_hash");
        let blocked: bool = row.get("blocked");
        let password = password.to_owned();
        let valid = tokio::task::spawn_blocking(move || {
            PasswordHash::new(&hash).is_ok_and(|h| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &h)
                    .is_ok()
            })
        })
        .await
        .map_err(|_| Failure("SERVICE_UNAVAILABLE", "验证服务暂不可用"))?;
        let mut s = Self::load(&mut tx).await?;
        let now = Self::time(&mut tx).await?;
        if !valid || blocked || !s.user(&actor).is_ok_and(|u| u.status == "active") {
            if !blocked {
                sqlx::query("UPDATE crowd_staff_accounts SET failed_attempts=failed_attempts+1,blocked_until=CASE WHEN failed_attempts>=4 THEN now()+interval '15 minutes' ELSE blocked_until END WHERE user_id=$1").bind(&actor).execute(&mut *tx).await.map_err(database)?;
            }
            s.log_failure(
                &actor,
                "staff_reauthenticate",
                &actor,
                request,
                &now,
                "AUTH_INVALID",
            );
            Self::save(&mut tx, &s).await?;
            Self::commit(tx).await?;
            return Err(Failure("AUTH_INVALID", "再次验证失败，或账号暂不可用"));
        }
        let version = s.security.policy["version"]
            .as_i64()
            .ok_or(Failure("POLICY_REQUIRED", "再次验证政策尚未发布"))?;
        sqlx::query("INSERT INTO crowd_session_reauthentication(token_hash,policy_version) VALUES(encode(digest($1,'sha256'),'hex'),$2) ON CONFLICT(token_hash) DO UPDATE SET verified_at=now(),policy_version=EXCLUDED.policy_version").bind(token).bind(version).execute(&mut *tx).await.map_err(database)?;
        sqlx::query(
            "UPDATE crowd_staff_accounts SET failed_attempts=0,blocked_until=NULL WHERE user_id=$1",
        )
        .bind(&actor)
        .execute(&mut *tx)
        .await
        .map_err(database)?;
        s.log(&actor, "staff_reauthenticate", &actor, request, &now);
        Self::save(&mut tx, &s).await?;
        Self::commit(tx).await?;
        Ok(
            json!({"verified":true,"policy_version":version,"expires_in":s.security.policy["reauth_seconds"]}),
        )
    }
    pub(crate) async fn require_reauthentication(
        tx: &mut Transaction<'_, Postgres>,
        s: &State,
        token: &str,
        action: &str,
    ) -> Result<()> {
        let p = &s.security.policy;
        if !p["actions"]
            .as_array()
            .is_some_and(|v| v.iter().any(|a| a == action))
        {
            return Ok(());
        }
        let ttl = p["reauth_seconds"]
            .as_i64()
            .ok_or(Failure("POLICY_REQUIRED", "再次验证政策缺失"))?;
        let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM crowd_session_reauthentication WHERE token_hash=encode(digest($1,'sha256'),'hex') AND policy_version=$2 AND verified_at + ($3::double precision * interval '1 second') > now())").bind(token).bind(p["version"].as_i64().unwrap_or(0)).bind(ttl).fetch_one(&mut **tx).await.map_err(database)?;
        if valid {
            Ok(())
        } else {
            Err(Failure(
                "REAUTH_REQUIRED",
                "请使用后台密码再次验证后继续原操作",
            ))
        }
    }
}
