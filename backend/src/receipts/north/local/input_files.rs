use crate::receipts::south::port::Repository;
use crate::shared::Result;
use serde_json::Value;
use sqlx::{Postgres, Transaction};
pub(crate) async fn validate_input_files(
    tx: &mut Transaction<'_, Postgres>,
    actor: &str,
    action: &str,
    input: &Value,
) -> Result<()> {
    if action == "receipt" && input["method"] == "wechat" {
        Repository::validate_file(
            tx,
            crate::shared::text(input, "file_id")?,
            actor,
            "wechat_receipt",
        )
        .await?;
    }
    Ok(())
}
