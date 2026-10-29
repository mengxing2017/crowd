use crate::catalog::south::port::Repository;
use crate::shared::{Failure, Result};
use serde_json::{Value, json};

#[derive(Clone)]
pub struct CollectTouch {
    repo: Repository,
}

impl CollectTouch {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &Value, key: &str, request: &str) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let mut state = Repository::load(&mut tx).await?;
        let idempotency = format!("guest-touch:{key}");
        if let Some(previous) = state.idempotency.get(&idempotency) {
            if previous["input"] != *input {
                return Err(Failure("IDEMPOTENCY_CONFLICT", "同一访问键内容不同"));
            }
            return Ok(previous["result"].clone());
        }
        let generated = Repository::entropy(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        let value = state.collect_touch(input, &generated, now)?;
        state
            .idempotency
            .insert(idempotency, json!({"input":input,"result":value}));
        let time = Repository::time(&mut tx).await?;
        state.log("guest", "REQ-05-001", "visit", request, &time);
        crate::catalog::south::port::reconcile_user_notices(&mut state, now);
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(value)
    }
}
