use crate::platform::domain::{Failure, Result, State};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Clone)]
pub struct Repository {
    pub pool: PgPool,
}
pub(crate) fn database(_: sqlx::Error) -> Failure {
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
        let mut state: State = serde_json::from_value(v)
            .map_err(|_| Failure("SERVICE_UNAVAILABLE", "业务记录暂不可读取"))?;
        crate::commissions::south::port::rules::sync_tasks(
            &mut state,
            "commission-reconciliation",
            Self::epoch(tx).await?,
        )?;
        Ok(state)
    }
    pub async fn save(tx: &mut Transaction<'_, Postgres>, state: &State) -> Result<()> {
        Self::save_code_ownership(tx, state).await?;
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

    pub async fn time(tx: &mut Transaction<'_, Postgres>) -> Result<String> {
        sqlx::query_scalar("SELECT to_char(clock_timestamp() AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"')").fetch_one(&mut **tx).await.map_err(database)
    }
}
