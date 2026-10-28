use crate::identity::south::port::IdentityProvider;
use crate::platform::{domain::Result, south::adapter::postgres::Repository};
use serde_json::Value;

/// Compatibility facade. New adapters depend on individual application use cases.
#[derive(Clone)]
pub struct Service {
    pub repo: Repository,
}

impl Service {
    pub async fn staff_login(&self, input: &Value, request: &str) -> Result<Value> {
        crate::identity::north::local::StaffLogin::new(self.repo.clone())
            .execute(input, request)
            .await
    }
    pub async fn login(
        &self,
        provider: &impl IdentityProvider,
        input: &Value,
        request: &str,
    ) -> Result<Value> {
        crate::identity::north::local::WechatLogin::new(self.repo.clone())
            .execute(provider, input, request)
            .await
    }
    pub async fn agreements(&self) -> Result<Value> {
        crate::governance::north::local::ReadAgreements::new(self.repo.clone())
            .execute()
            .await
    }
    pub async fn resolve(&self, code: &str) -> Result<Value> {
        crate::promotion::north::local::ResolvePromotion::new(self.repo.clone())
            .execute(code)
            .await
    }
    pub async fn read(
        &self,
        token: &str,
        section: &str,
        target: Option<&str>,
        request: &str,
    ) -> Result<Value> {
        crate::platform::north::local::ReadResource::new(self.repo.clone())
            .execute(token, section, target, request)
            .await
    }
    pub async fn command(
        &self,
        token: &str,
        action: &str,
        input: &Value,
        key: &str,
        request: &str,
    ) -> Result<Value> {
        crate::platform::north::local::ExecuteCommand::new(self.repo.clone())
            .execute(token, action, input, key, request)
            .await
    }
    pub async fn upload(
        &self,
        token: &str,
        purpose: &str,
        bytes: Vec<u8>,
        request: &str,
    ) -> Result<Value> {
        crate::files::north::local::UploadFile::new(self.repo.clone())
            .execute(token, purpose, bytes, request)
            .await
    }
    pub async fn file(&self, token: &str, id: &str, request: &str) -> Result<(String, Vec<u8>)> {
        crate::files::north::local::ReadPrivateFile::new(self.repo.clone())
            .execute(token, id, request)
            .await
    }
    pub async fn reconcile_catalog(&self) -> Result<()> {
        crate::catalog::north::local::ReconcileCatalog::new(self.repo.clone())
            .execute()
            .await
    }
    pub async fn public_catalog(&self, query: &Value) -> Result<Value> {
        crate::catalog::north::local::ReadPublicCatalog::new(self.repo.clone())
            .execute(query)
            .await
    }
    pub async fn touch(&self, input: &Value, key: &str, request: &str) -> Result<Value> {
        crate::catalog::north::local::CollectTouch::new(self.repo.clone())
            .execute(input, key, request)
            .await
    }
    pub async fn validate_catalog_files(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        state: &crate::platform::domain::State,
        actor: &str,
        action: &str,
        input: &Value,
    ) -> Result<()> {
        crate::catalog::north::local::ValidateProductFiles::execute(tx, state, actor, action, input)
            .await
    }
    pub async fn product_image(&self, id: &str) -> Result<(String, Vec<u8>)> {
        crate::catalog::north::local::ReadProductImage::new(self.repo.clone())
            .execute(id)
            .await
    }
    pub async fn record_verified_completion(
        &self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
        request: &str,
    ) -> Result<Value> {
        crate::catalog::north::local::RecordVerifiedCompletion::new(self.repo.clone())
            .execute(source, event, claim, occurred_at, request)
            .await
    }
    pub async fn record_verified_task_result(
        &self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
        base_cents: Option<i64>,
        request: &str,
    ) -> Result<Value> {
        crate::tasks::north::local::RecordVerifiedTaskResult::new(self.repo.clone())
            .execute(source, event, claim, occurred_at, base_cents, request)
            .await
    }
    pub async fn evidence_file(
        &self,
        token: &str,
        id: &str,
        reason: &str,
        request: &str,
    ) -> Result<(String, Vec<u8>)> {
        crate::tasks::north::local::ReadEvidenceFile::new(self.repo.clone())
            .execute(token, id, reason, request)
            .await
    }
}
