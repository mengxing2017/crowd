use super::command_context::CommandContext;
use crate::community::{
    domain::{Failure, Result, State},
    south::adapter::postgres::Repository,
};
use serde_json::{Value, json};
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
    Ok::<Value, Failure>(if action == "logout" {
        Repository::logout(tx, token).await?;
        json!({"status":"logged_out"})
    } else if action == "web_ticket" {
        Repository::web_ticket(tx, actor).await?
    } else if action == "receipt_reveal" {
        crate::community::domain::text(input, "reason")?;
        let id = crate::community::domain::text(input, "id")?;
        let receipt = state
            .user(actor)?
            .receipts
            .iter()
            .find(|v| v["id"] == id)
            .cloned()
            .ok_or(Failure("NOT_FOUND", "资料不存在"))?;
        state.log(actor, "REQ-03-008", id, request, now);
        receipt
    } else if crate::tasks::domain::is_command(action) {
        crate::tasks::north::local::apply_command(tx, state, context).await?
    } else if crate::catalog::domain::is_command(action) {
        crate::catalog::north::local::apply_command(state, context)?
    } else {
        state.command(actor, action, input, generated, request, now)?
    })
}
