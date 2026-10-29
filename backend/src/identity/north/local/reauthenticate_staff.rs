use crate::identity::south::port::Repository;
use crate::shared::Result;
use serde_json::Value;
#[derive(Clone)]
pub struct ReauthenticateStaff {
    repo: Repository,
}
impl ReauthenticateStaff {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
    pub async fn execute(&self, token: &str, password: &str, request: &str) -> Result<Value> {
        self.repo.reauthenticate(token, password, request).await
    }
}
