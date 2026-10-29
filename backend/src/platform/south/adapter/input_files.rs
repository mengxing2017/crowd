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
    if action == "ticket_reply" {
        for id in input["attachments"].as_array().into_iter().flatten() {
            let id = id
                .as_str()
                .ok_or(crate::shared::Failure("INVALID_INPUT", "附件编号无效"))?;
            crate::platform::south::adapter::postgres::Repository::validate_file(
                tx,
                id,
                actor,
                "support_attachment",
            )
            .await?;
        }
    }
    Ok(())
}
