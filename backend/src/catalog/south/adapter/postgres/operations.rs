use crate::platform::south::adapter::postgres::{Repository, database};
use crate::shared::{Failure, Result};

use sqlx::{Postgres, Transaction};
impl Repository {
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
}
