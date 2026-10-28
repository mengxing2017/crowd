use crate::community::domain::{Failure, Result};
use crate::community::south::adapter::postgres::Repository;
use serde_json::{Value, json};

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
        if let Some(id) = task_id {
            let t = state
                .tasks
                .instances
                .get(id)
                .ok_or(Failure("NOT_FOUND", "任务不存在"))?;
            if t["owner"] != actor
                || !matches!(t["status"].as_str(), Some("in_progress" | "supplement"))
            {
                return Err(Failure("FORBIDDEN", "不能向该任务上传证据"));
            }
            if size > t["template"]["max_file_bytes"].as_u64().unwrap_or(0) as usize {
                return Err(Failure("INVALID_INPUT", "证据图片超过模板大小限制"));
            }
            let count = state
                .tasks
                .attachments
                .values()
                .filter(|a| a["task_id"] == id)
                .count();
            if count >= 100 {
                return Err(Failure(
                    "INVALID_INPUT",
                    "当前任务上传文件已达服务资源上限，请联系客服",
                ));
            }
        } else if purpose == "task_evidence" {
            return Err(Failure("INVALID_INPUT", "证据上传必须绑定任务"));
        }
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
        if let Some(id) = task_id {
            state.tasks.attachments.insert(
                result["id"].as_str().unwrap().into(),
                json!({"task_id":id,"owner":actor,"size":size}),
            );
        }
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
