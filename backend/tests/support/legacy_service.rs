use crate::identity::south::port::IdentityProvider;
use crate::platform::south::port::Repository;
use crate::shared::Result;
use serde_json::Value;

/// Compatibility facade. New adapters depend on individual application use cases.
#[derive(Clone)]
pub struct Service {
    pub repo: Repository,
}

impl Service {
    pub async fn staff_login(&self, input: &Value, request: &str) -> Result<Value> {
        crate::identity::south::port::services::StaffLogin::new(self.repo.clone())
            .execute(input, request)
            .await
    }
    pub async fn login(
        &self,
        provider: &impl IdentityProvider,
        input: &Value,
        request: &str,
    ) -> Result<Value> {
        crate::identity::south::port::services::WechatLogin::new(self.repo.clone())
            .execute(provider, input, request)
            .await
    }

    pub async fn read(
        &self,
        token: &str,
        section: &str,
        target: Option<&str>,
        request: &str,
    ) -> Result<Value> {
        crate::platform::south::port::services::ReadResource::new(self.repo.clone())
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
        crate::platform::south::port::services::ExecuteCommand::new(self.repo.clone())
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
        crate::files::south::port::services::UploadFile::new(self.repo.clone())
            .execute(token, purpose, bytes, request)
            .await
    }
    pub async fn file(&self, token: &str, id: &str, request: &str) -> Result<(String, Vec<u8>)> {
        crate::files::south::port::services::ReadPrivateFile::new(self.repo.clone())
            .execute(token, id, request)
            .await
    }

    pub async fn public_catalog(&self, query: &Value) -> Result<Value> {
        crate::catalog::south::port::services::ReadPublicCatalog::new(self.repo.clone())
            .execute(query)
            .await
    }
    pub async fn touch(&self, input: &Value, key: &str, request: &str) -> Result<Value> {
        crate::catalog::south::port::services::CollectTouch::new(self.repo.clone())
            .execute(input, key, request)
            .await
    }

    pub async fn product_image(&self, id: &str) -> Result<(String, Vec<u8>)> {
        crate::catalog::south::port::services::ReadProductImage::new(self.repo.clone())
            .execute(id)
            .await
    }

    pub async fn evidence_file(
        &self,
        token: &str,
        id: &str,
        reason: &str,
        request: &str,
    ) -> Result<(String, Vec<u8>)> {
        crate::tasks::south::port::services::ReadEvidenceFile::new(self.repo.clone())
            .execute(token, id, reason, request)
            .await
    }
}
