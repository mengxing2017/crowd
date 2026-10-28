use crate::platform::south::port::Repository;
use crate::shared::Result;
use sqlx::PgPool;

/// Explicit startup integration. Does not expose the transaction document or repository.
pub async fn bootstrap_staff(pool: PgPool, username: &str, password: &str) -> Result<bool> {
    Repository { pool }
        .bootstrap_staff(username, password)
        .await
}
