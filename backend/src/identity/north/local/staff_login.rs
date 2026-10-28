use crate::platform::domain::Result;
use crate::platform::south::adapter::postgres::Repository;
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
        let username = crate::platform::domain::text(input, "username")?;
        let password = crate::platform::domain::text(input, "password")?;
        self.repo.staff_login(username, password, request).await
    }
}
