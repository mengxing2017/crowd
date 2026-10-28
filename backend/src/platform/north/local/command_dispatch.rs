use super::command_context::CommandContext;
use crate::platform::domain::{Failure, Result, State};
use serde_json::Value;
use sqlx::{Postgres, Transaction};

pub(super) async fn apply(
    tx: &mut Transaction<'_, Postgres>,
    state: &mut State,
    token: &str,
    context: &CommandContext<'_>,
) -> Result<Value> {
    let CommandContext {
        actor,
        action,
        input,
        generated,
        request,
        now,
        epoch: _,
    } = *context;
    Ok::<Value, Failure>(if matches!(action, "logout" | "web_ticket") {
        crate::identity::north::local::session_command(tx, token, actor, action).await?
    } else if action == "receipt_cleanup" {
        crate::receipts::north::local::cleanup(tx, state, context).await?
    } else if action == "receipt_reveal" {
        crate::receipts::north::local::reveal(state, actor, input, request, now)?
    } else if crate::commissions::domain::is_command(action) {
        crate::commissions::north::local::apply(state, context)?
    } else if crate::withdrawals::domain::is_command(action) {
        if matches!(action, "withdrawal_preview" | "withdrawal_apply") {
            let u = state.user(actor)?;
            if let Some(r) = u
                .receipts
                .iter()
                .find(|r| r["id"] == input["receipt_id"] && r["method"] == "wechat")
            {
                crate::platform::south::adapter::postgres::Repository::validate_file(
                    tx,
                    crate::shared::text(r, "file_id")?,
                    actor,
                    "wechat_receipt",
                )
                .await?;
            }
        }
        crate::withdrawals::north::local::apply(state, context)?
    } else if crate::settlements::domain::is_command(action) {
        if action == "settlement_register" {
            crate::platform::south::adapter::postgres::Repository::validate_file(
                tx,
                crate::shared::text(input, "voucher_file_id")?,
                actor,
                "settlement_voucher",
            )
            .await?;
        }
        crate::settlements::north::local::apply(state, context)?
    } else if crate::tasks::domain::is_command(action) {
        crate::tasks::north::local::apply_command(tx, state, context).await?
    } else if crate::catalog::domain::is_command(action) {
        crate::catalog::north::local::apply_command(state, context)?
    } else {
        state.command(actor, action, input, generated, request, now)?
    })
}
