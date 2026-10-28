use crate::platform::domain::{Failure, Result};
use crate::platform::south::adapter::postgres::Repository;
use serde_json::Value;

#[derive(Clone)]
pub struct ReadAgreements {
    repo: Repository,
}

impl ReadAgreements {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let state = Repository::load(&mut tx).await?;
        let result = state
            .rules
            .iter()
            .find(|r| r["kind"] == "agreements" && r["status"] == "published")
            .cloned()
            .ok_or(Failure(
                "AGREEMENT_UNAVAILABLE",
                "服务协议与隐私说明尚未发布",
            ))?;
        Ok(result)
    }
}
