use crate::platform::south::adapter::postgres::Repository;
use crate::shared::Result;
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};
pub(crate) async fn session_command(
    tx: &mut Transaction<'_, Postgres>,
    token: &str,
    actor: &str,
    action: &str,
) -> Result<Value> {
    if action == "logout" {
        Repository::logout(tx, token).await?;
        Ok(json!({"status":"logged_out"}))
    } else {
        Repository::web_ticket(tx, actor).await
    }
}
