use crate::platform::domain::{Result, State};
use serde_json::Value;
use sqlx::{Postgres, Transaction};

pub(crate) async fn validate(
    tx: &mut Transaction<'_, Postgres>,
    state: &State,
    actor: &str,
    action: &str,
    input: &Value,
) -> Result<()> {
    crate::identity::south::port::services::validate_input_files(tx, actor, action, input).await?;
    crate::receipts::south::port::services::validate_input_files(tx, actor, action, input).await?;
    crate::catalog::south::port::services::ValidateProductFiles::execute(
        tx, state, actor, action, input,
    )
    .await?;
    Ok(())
}
