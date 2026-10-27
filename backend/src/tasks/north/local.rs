use crate::{
    community::{
        domain::{Failure, Result},
        north::local::Service,
        south::adapter::postgres::Repository,
    },
    tasks::south::port::VerifiedTaskResult,
};
use serde_json::Value;
impl Service {
    /// Internal adapter entry only: source authentication is the adapter's responsibility.
    pub async fn record_verified_task_result(
        &self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
        base_cents: Option<i64>,
        request: &str,
    ) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let mut state = Repository::load(&mut tx).await?;
        if occurred_at > Repository::epoch(&mut tx).await? {
            return Err(Failure("INVALID_INPUT", "可信成果时间不能在未来"));
        }
        let value = state.record_task_result(source, event, claim, occurred_at, base_cents)?;
        let now = Repository::time(&mut tx).await?;
        state.log(source, "verified_task_result", claim, request, &now);
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(value)
    }
}
