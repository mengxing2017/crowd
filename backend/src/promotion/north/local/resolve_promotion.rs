use crate::platform::domain::Result;
use crate::platform::south::adapter::postgres::Repository;
use serde_json::Value;

#[derive(Clone)]
pub struct ResolvePromotion {
    repo: Repository,
}

impl ResolvePromotion {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, code: &str) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        Repository::load(&mut tx).await?.resolve(code)
    }
}
