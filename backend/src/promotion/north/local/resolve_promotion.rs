use crate::promotion::south::port::Repository;
use crate::shared::Result;
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
