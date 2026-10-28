use crate::community::{
    domain::{Result, State},
    south::adapter::postgres::Repository,
};
use serde_json::Value;
use sqlx::{Postgres, Transaction};

pub(super) async fn validate(
    tx: &mut Transaction<'_, Postgres>,
    state: &State,
    actor: &str,
    action: &str,
    input: &Value,
) -> Result<()> {
    if action == "profile"
        && let Some(id) = input["avatar"].as_str()
    {
        Repository::validate_file(tx, id, actor, "avatar").await?;
    }
    if action == "receipt" && input["method"] == "wechat" {
        Repository::validate_file(
            tx,
            crate::community::domain::text(input, "file_id")?,
            actor,
            "wechat_receipt",
        )
        .await?;
    }
    crate::catalog::north::local::ValidateProductFiles::execute(tx, state, actor, action, input)
        .await?;
    Ok(())
}
