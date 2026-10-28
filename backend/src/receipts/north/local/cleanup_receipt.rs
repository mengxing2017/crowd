use crate::receipts::south::port::DomainPort;
use crate::{
    receipts::south::port::{CommandContext, Repository},
    shared::Result,
    shared::text,
};
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};

pub(crate) async fn cleanup<P: DomainPort + ?Sized>(
    tx: &mut Transaction<'_, Postgres>,
    s: &mut P,
    c: &CommandContext<'_>,
) -> Result<Value> {
    let (file, seconds) = crate::receipts::domain::cleanup_candidate(s, c.actor, c.input)?;
    Repository::delete_expired_receipt(tx, c.actor, &file, seconds).await?;
    let id = text(c.input, "id")?;
    let version = s.user(c.actor)?.version;
    let version = s.append_receipt_event(
        c.actor,
        version,
        json!({"id":c.generated,"cleans":id,"status":"cleaned","time":c.now,"file_id":file}),
    )?;
    s.log(c.actor, "REQ-17-010", id, c.request, c.now);
    {
        let mut log = serde_json::json!({});
        log["reason"] = c.input["reason"].clone();
        log["retention_seconds"] = json!(seconds);

        s.annotate_audit(log);
    }
    Ok(json!({"id":id,"status":"cleaned","version":version}))
}
