use crate::platform::south::port::database;
use crate::shared::{Failure, Result, require};
use crate::tasks::south::port::Repository;
use sqlx::{Postgres, Transaction};
/// HMAC over the exact UTF-8 body; secrets are deployment configuration, never business data.
pub(crate) async fn verify(
    tx: &mut Transaction<'_, Postgres>,
    source: &str,
    signature: &str,
    body: &[u8],
) -> Result<()> {
    require(
        !source.is_empty()
            && source.len() <= 64
            && source
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_'),
        "来源编号无效",
    )?;
    let key = std::env::var(format!(
        "CROWD_TASK_SOURCE_{}_SECRET",
        source.to_ascii_uppercase()
    ))
    .ok()
    .filter(|v| v.len() >= 32)
    .ok_or(Failure("SOURCE_DISABLED", "任务来源未启用"))?;
    let expected: String =
        sqlx::query_scalar("SELECT encode(hmac($1::bytea,$2::bytea,'sha256'),'hex')")
            .bind(body)
            .bind(key.as_bytes())
            .fetch_one(&mut **tx)
            .await
            .map_err(database)?;
    let mut diff = expected.len() ^ signature.len();
    for (a, b) in expected.bytes().zip(signature.bytes()) {
        diff |= (a ^ b) as usize;
    }
    require(diff == 0, "任务事件签名无效")
}
pub(crate) use crate::tasks::north::event::ReceiveSignedTaskEvent;
pub(crate) async fn received_at(tx: &mut Transaction<'_, Postgres>) -> Result<i64> {
    Repository::epoch(tx).await
}

pub(crate) async fn payload_hash(
    tx: &mut Transaction<'_, Postgres>,
    body: &[u8],
) -> Result<String> {
    sqlx::query_scalar("SELECT encode(digest($1::bytea,'sha256'),'hex')")
        .bind(body)
        .fetch_one(&mut **tx)
        .await
        .map_err(database)
}
