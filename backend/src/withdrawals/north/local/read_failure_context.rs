use crate::{platform::south::adapter::postgres::Repository, shared::Result};
use serde_json::{Value, json};
#[derive(Clone)]
pub struct ReadWithdrawalFailureContext {
    repo: Repository,
}
impl ReadWithdrawalFailureContext {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
    pub async fn execute(&self, token: &str, request: &str) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        let summary =
            crate::commissions::domain::read(&state, &actor, "commission_summary", &actor)?;
        let now = Repository::epoch(&mut tx).await?;
        let apps = crate::withdrawals::domain::read(&state, &actor, "my_withdrawals", &actor, now)?;
        state.log(
            &actor,
            "withdrawal_failure_context",
            &actor,
            request,
            &Repository::time(&mut tx).await?,
        );
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(json!({"summary":summary,"applications":apps}))
    }
}
