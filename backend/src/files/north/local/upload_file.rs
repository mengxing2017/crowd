use crate::platform::domain::{Failure, Result};
use crate::platform::south::adapter::postgres::Repository;
use serde_json::Value;

#[derive(Clone)]
pub struct UploadFile {
    repo: Repository,
}

impl UploadFile {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(
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
        if purpose == "product_image" {
            state.authorize(&actor, "operator", "platform")?;
        }
        let task_id = purpose.strip_prefix("task_evidence:");
        let size = bytes.len();
        crate::tasks::domain::validate_upload(&state, &actor, purpose, size)?;
        let result = Repository::upload(
            &mut tx,
            &actor,
            if task_id.is_some() {
                "task_evidence"
            } else {
                purpose
            },
            bytes,
        )
        .await?;
        crate::tasks::domain::register_upload(
            &mut state,
            &actor,
            task_id,
            result["id"].as_str().unwrap(),
            size,
        );
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
}
