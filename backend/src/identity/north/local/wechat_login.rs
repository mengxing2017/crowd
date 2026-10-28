use crate::identity::south::port::IdentityProvider;
use crate::platform::domain::{Failure, Result};
use crate::platform::south::adapter::postgres::Repository;
use serde_json::Value;

#[derive(Clone)]
pub struct WechatLogin {
    repo: Repository,
}

impl WechatLogin {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        provider: &impl IdentityProvider,
        input: &Value,
        request: &str,
    ) -> Result<Value> {
        let code = crate::platform::domain::text(input, "code")?;
        let platform = provider.exchange(code).await.map_err(|_| {
            Failure(
                "IDENTITY_UNAVAILABLE",
                "微信凭证校验失败或身份服务暂不可用，请重新登录",
            )
        })?;
        self.repo.login(&platform, input, request).await
    }
}
