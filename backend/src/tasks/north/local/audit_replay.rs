use crate::platform::{
    domain::{Result, State},
    south::adapter::postgres::Repository,
};
use serde_json::Value;
use sqlx::{Postgres, Transaction};

pub(crate) async fn audit_replay(
    tx: &mut Transaction<'_, Postgres>,
    state: &mut State,
    actor: &str,
    action: &str,
    input: &Value,
    request: &str,
) -> Result<bool> {
    if action != "evidence_reveal" {
        return Ok(false);
    }
    let now = Repository::time(tx).await?;
    state.log(
        actor,
        "evidence_reveal",
        input["id"].as_str().unwrap_or(""),
        request,
        &now,
    );
    if let Some(log) = state.audit.last_mut() {
        log["reason"] = input["reason"].clone();
    }
    Ok(true)
}
