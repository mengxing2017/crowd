use crate::community::domain::{Failure, Result};
use crate::community::south::adapter::postgres::Repository;
use serde_json::Value;

#[derive(Clone)]
pub struct RecordVerifiedCompletion {
    repo: Repository,
}

impl RecordVerifiedCompletion {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
        request: &str,
    ) -> Result<Value> {
        use crate::catalog::south::port::CompletionRecorder;
        let mut tx = self.repo.begin().await?;
        let mut state = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        if occurred_at > now {
            return Err(Failure("INVALID_INPUT", "完成事件时间不能晚于服务端时间"));
        }
        let value = state.record_completion(source, event, claim, occurred_at)?;
        let time = Repository::time(&mut tx).await?;
        state.log(source, "REQ-05-010", claim, request, &time);
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(value)
    }
}
