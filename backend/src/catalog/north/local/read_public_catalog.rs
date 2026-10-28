use crate::community::domain::Result;
use crate::community::south::adapter::postgres::Repository;
use serde_json::Value;

#[derive(Clone)]
pub struct ReadPublicCatalog {
    repo: Repository,
}

impl ReadPublicCatalog {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, query: &Value) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let state = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        state.public_catalog(query, now)
    }
}
