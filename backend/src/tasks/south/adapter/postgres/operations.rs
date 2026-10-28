use crate::platform::south::adapter::postgres::{Repository, database};
use crate::shared::Result;

use sqlx::{Postgres, Transaction};
impl Repository {
    pub async fn delete_unused_evidence(
        tx: &mut Transaction<'_, Postgres>,
        id: &str,
        owner: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM crowd_files WHERE id=$1 AND owner_id=$2 AND purpose='task_evidence'",
        )
        .bind(id)
        .bind(owner)
        .execute(&mut **tx)
        .await
        .map_err(database)?;
        Ok(())
    }
}
