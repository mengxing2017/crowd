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
    } else if action == "receipt_reveal" {
        crate::receipts::north::local::reveal(state, actor, input, request, now)?
    } else if crate::tasks::domain::is_command(action) {
        crate::tasks::north::local::apply_command(tx, state, context).await?
    } else if crate::catalog::domain::is_command(action) {
        crate::catalog::north::local::apply_command(state, context)?
    } else {
        state.command(actor, action, input, generated, request, now)?
    })
}
