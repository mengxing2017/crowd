use crate::community::domain::Result;
use crate::community::south::adapter::postgres::Repository;
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
        let username = crate::community::domain::text(input, "username")?;
        let password = crate::community::domain::text(input, "password")?;
        self.repo.staff_login(username, password, request).await
    }
}
