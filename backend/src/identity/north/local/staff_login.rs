use crate::identity::south::port::Repository;
use crate::shared::Result;
use serde_json::Value;

#[derive(Clone)]
pub struct StaffLogin {
    repo: Repository,
}

impl StaffLogin {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &Value, request: &str) -> Result<Value> {
        let username = crate::shared::text(input, "username")?;
        let password = crate::shared::text(input, "password")?;
        self.repo.staff_login(username, password, request).await
    }
}
