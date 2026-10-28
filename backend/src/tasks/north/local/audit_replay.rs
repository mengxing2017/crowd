use crate::shared::Result;
use crate::tasks::south::port::DomainPort;
use crate::tasks::south::port::Repository;
use serde_json::Value;
use sqlx::{Postgres, Transaction};

pub(crate) async fn audit_replay<P: DomainPort + ?Sized>(
    tx: &mut Transaction<'_, Postgres>,
    state: &mut P,
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
    {
        let mut log = serde_json::json!({});
        log["reason"] = input["reason"].clone();

        state.annotate_audit(log);
    }
    Ok(true)
}
