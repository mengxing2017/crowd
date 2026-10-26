use crate::community::{
    domain::{Failure, Result},
    south::{adapter::postgres::Repository, port::IdentityProvider},
};
use serde_json::{Value, json};
#[derive(Clone)]
pub struct Service {
    pub repo: Repository,
}
impl Service {
    pub async fn staff_login(&self, input: &Value, request: &str) -> Result<Value> {
        let username = crate::community::domain::text(input, "username")?;
        let password = crate::community::domain::text(input, "password")?;
        self.repo.staff_login(username, password, request).await
    }

    pub async fn login(
        &self,
        provider: &impl IdentityProvider,
        input: &Value,
        request: &str,
    ) -> Result<Value> {
        let code = crate::community::domain::text(input, "code")?;
        let platform = provider.exchange(code).await.map_err(|_| {
            Failure(
                "IDENTITY_UNAVAILABLE",
                "微信凭证校验失败或身份服务暂不可用，请重新登录",
            )
        })?;
        self.repo.login(&platform, input, request).await
    }
    pub async fn agreements(&self) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let state = Repository::load(&mut tx).await?;
        let result = state
            .rules
            .iter()
            .find(|r| r["kind"] == "agreements" && r["status"] == "published")
            .cloned()
            .ok_or(Failure(
                "AGREEMENT_UNAVAILABLE",
                "服务协议与隐私说明尚未发布",
            ))?;
        Ok(result)
    }
    pub async fn resolve(&self, code: &str) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        Repository::load(&mut tx).await?.resolve(code)
    }
    pub async fn read(
        &self,
        token: &str,
        section: &str,
        target: Option<&str>,
        request: &str,
    ) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        let now = Repository::time(&mut tx).await?;
        let result = state.read(&actor, section, target.unwrap_or(&actor), request, &now);
        if let Err(ref failure) = result {
            state.log_failure(
                &actor,
                section,
                target.unwrap_or(&actor),
                request,
                &now,
                failure.0,
            );
        }
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        result
    }
    pub async fn command(
        &self,
        token: &str,
        action: &str,
        input: &Value,
        key: &str,
        request: &str,
    ) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        state.check_command_access(&actor, action, input)?;
        let idempotency = format!("{actor}:{key}");
        if let Some(previous) = state.idempotency.get(&idempotency) {
            if previous["action"] != action || previous["input"] != *input {
                return Err(Failure("IDEMPOTENCY_CONFLICT", "同一请求键内容不同"));
            }
            return Ok(previous["result"].clone());
        }
        if action == "profile"
            && let Some(id) = input["avatar"].as_str()
        {
            Repository::validate_file(&mut tx, id, &actor, "avatar").await?;
        }
        if action == "receipt" && input["method"] == "wechat" {
            Repository::validate_file(
                &mut tx,
                crate::community::domain::text(input, "file_id")?,
                &actor,
                "wechat_receipt",
            )
            .await?;
        }
        let generated = Repository::entropy(&mut tx).await?;
        let now = Repository::time(&mut tx).await?;
        let before = state.clone();
        let result = async {
            Ok::<Value, Failure>(if action == "logout" {
                Repository::logout(&mut tx, token).await?;
                json!({"status":"logged_out"})
            } else if action == "web_ticket" {
                Repository::web_ticket(&mut tx, &actor).await?
            } else if action == "receipt_reveal" {
                crate::community::domain::text(input, "reason")?;
                let id = crate::community::domain::text(input, "id")?;
                let receipt = state
                    .user(&actor)?
                    .receipts
                    .iter()
                    .find(|v| v["id"] == id)
                    .cloned()
                    .ok_or(Failure("NOT_FOUND", "资料不存在"))?;
                state.log(&actor, "REQ-03-008", id, request, &now);
                receipt
            } else {
                state.command(&actor, action, input, &generated, request, &now)?
            })
        }
        .await;
        let result = match result {
            Ok(value) => value,
            Err(failure) => {
                state = before;
                state.log_failure(&actor, action, &actor, request, &now, failure.0);
                Repository::save(&mut tx, &state).await?;
                Repository::commit(tx).await?;
                return Err(failure);
            }
        };
        if action == "logout" || action == "web_ticket" {
            state.log(&actor, action, &actor, request, &now);
        }
        state.idempotency.insert(
            idempotency,
            json!({"action":action,"input":input,"result":result}),
        );
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(result)
    }
    pub async fn upload(
        &self,
        token: &str,
        purpose: &str,
        bytes: Vec<u8>,
        request: &str,
    ) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        if state.user(&actor)?.status != "active" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号当前不可修改资料"));
        }
        let result = Repository::upload(&mut tx, &actor, purpose, bytes).await?;
        let now = Repository::time(&mut tx).await?;
        state.log(
            &actor,
            "file_upload",
            result["id"].as_str().unwrap(),
            request,
            &now,
        );
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(result)
    }
    pub async fn file(&self, token: &str, id: &str, request: &str) -> Result<(String, Vec<u8>)> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        state.user(&actor)?;
        let result = Repository::file(&mut tx, id, &actor).await?;
        let now = Repository::time(&mut tx).await?;
        state.log(&actor, "file_view", id, request, &now);
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(result)
    }
}
