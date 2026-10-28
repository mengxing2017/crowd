use crate::identity::south::port::Repository;
use crate::shared::{Result, text};
use serde_json::Value;

#[derive(Clone)]
pub struct RedeemSession {
    repo: Repository,
}
impl RedeemSession {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
    pub async fn execute(&self, input: &Value) -> Result<Value> {
        self.repo.redeem(text(input, "ticket")?).await
    }
}
