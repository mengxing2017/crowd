use crate::privacy::south::port::{DomainPort, Repository};
use crate::shared::{CommandContext, Result};
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};
pub(crate) async fn cleanup_file<P: DomainPort + ?Sized>(
    tx: &mut Transaction<'_, Postgres>,
    s: &mut P,
    c: &CommandContext<'_>,
) -> Result<Value> {
    let seconds = crate::privacy::domain::cleanup_candidate(s, c.input)?;
    let id = crate::shared::text(c.input, "id")?;
    let purpose = crate::shared::text(c.input, "purpose")?;
    Repository::cleanup_private_file(tx, c.actor, id, purpose, seconds).await?;
    s.log(c.actor, "privacy_file_cleanup", id, c.request, c.now);
    Ok(json!({"id":id,"status":"cleaned","purpose":purpose,"retention_seconds":seconds}))
}
