use crate::platform::domain::{Result, State};
use serde_json::Value;
use sqlx::{Postgres, Transaction};

pub(super) async fn validate(
    tx: &mut Transaction<'_, Postgres>,
    state: &State,
    actor: &str,
    action: &str,
    input: &Value,
) -> Result<()> {
    crate::identity::north::local::validate_input_files(tx, actor, action, input).await?;
    crate::receipts::north::local::validate_input_files(tx, actor, action, input).await?;
    crate::catalog::north::local::ValidateProductFiles::execute(tx, state, actor, action, input)
        .await?;
    Ok(())
}
