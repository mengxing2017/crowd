use crate::platform::south::port::{Repository, database};
use crate::shared::{Failure, Result};
use sqlx::{Postgres, Transaction};
impl Repository {
    pub(crate) async fn cleanup_private_file(
        tx: &mut Transaction<'_, Postgres>,
        a: &str,
        id: &str,
        purpose: &str,
        seconds: i64,
    ) -> Result<()> {
        let result=sqlx::query("DELETE FROM crowd_files WHERE id=$1 AND owner_id=$2 AND purpose=$3 AND created_at<=now()-($4::double precision*interval '1 second')").bind(id).bind(a).bind(purpose).bind(seconds).execute(&mut **tx).await.map_err(database)?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(Failure(
                "RETENTION_PROTECTED",
                "文件用途、归属或保留期不允许清理",
            ))
        }
    }
}
