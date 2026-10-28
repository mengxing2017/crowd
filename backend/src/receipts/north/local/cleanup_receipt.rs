use crate::{
    platform::{
        domain::{Result, State},
        north::local::CommandContext,
        south::adapter::postgres::Repository,
    },
    shared::text,
};
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};

pub(crate) async fn cleanup(
    tx: &mut Transaction<'_, Postgres>,
    s: &mut State,
    c: &CommandContext<'_>,
) -> Result<Value> {
    let (file, seconds) = crate::receipts::domain::cleanup_candidate(s, c.actor, c.input)?;
    Repository::delete_expired_receipt(tx, c.actor, &file, seconds).await?;
    let u = s.users.get_mut(c.actor).unwrap();
    u.version += 1;
    let id = text(c.input, "id")?;
    u.receipts
        .push(json!({"id":c.generated,"cleans":id,"status":"cleaned","time":c.now,"file_id":file}));
    let version = u.version;
    s.log(c.actor, "REQ-17-010", id, c.request, c.now);
    if let Some(log) = s.audit.last_mut() {
        log["reason"] = c.input["reason"].clone();
        log["retention_seconds"] = json!(seconds);
    }
    Ok(json!({"id":id,"status":"cleaned","version":version}))
}
