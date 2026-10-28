use crate::platform::south::adapter::postgres::{Repository, database};
use crate::shared::{Failure, Result};
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};
impl Repository {
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
}
