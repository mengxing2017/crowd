use crate::notifications::south::port::{DeliveryOutcome, NotificationDelivery, Repository};
use crate::shared::{Failure, Result};
use serde_json::Value;
#[derive(Clone)]
pub struct DeliverNotification<P> {
    repo: Repository,
    provider: P,
}
impl<P: NotificationDelivery> DeliverNotification<P> {
    pub fn new(pool: sqlx::PgPool, provider: P) -> Self {
        Self {
            repo: Repository { pool },
            provider,
        }
    }
    pub async fn execute(&self, id: &str) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let mut state = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        let attempt = Repository::entropy(&mut tx).await?;
        let message =
            crate::notifications::domain::delivery::reserve(&mut state, id, &attempt, now)?;
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.provider.send(
                id,
                message["owner"].as_str().unwrap_or(""),
                message["template_id"].as_str().unwrap_or(""),
                message["body"].as_str().unwrap_or(""),
            ),
        )
        .await
        .unwrap_or(DeliveryOutcome::TemporaryFailure);
        let mut tx = self.repo.begin().await?;
        let mut state = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        let result =
            crate::notifications::domain::delivery::finish(&mut state, id, &attempt, outcome, now)?;
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        if result["external_status"] == "sending" {
            return Err(Failure("SERVICE_UNAVAILABLE", "通知结果待核查"));
        }
        Ok(result)
    }
}
