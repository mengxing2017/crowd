use crate::platform::domain::{Failure, Result};
use crate::platform::south::adapter::postgres::Repository;
use serde_json::json;

#[derive(Clone)]
pub struct ReadEvidenceFile {
    repo: Repository,
}

impl ReadEvidenceFile {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        token: &str,
        id: &str,
        reason: &str,
        request: &str,
    ) -> Result<(String, Vec<u8>)> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        crate::platform::domain::text(&json!({"reason":reason}), "reason")?;
        let a = state
            .tasks
            .attachments
            .get(id)
            .ok_or(Failure("NOT_FOUND", "证据图片不存在"))?;
        let t = state
            .tasks
            .instances
            .get(a["task_id"].as_str().unwrap_or(""))
            .ok_or(Failure("NOT_FOUND", "任务不存在"))?;
        if state.user(&actor)?.status != "active" {
            return Err(Failure("FORBIDDEN", "账号不可查看证据"));
        }
        if a["owner"] != actor {
            let target = t["product_id"].as_str().unwrap_or("");
            let reviewer = state.authorize(&actor, "task_reviewer", target).is_ok()
                && t["reviewer"] == actor
                && matches!(
                    t["status"].as_str(),
                    Some("reviewing" | "awaiting_verification")
                );
            if !reviewer && state.authorize(&actor, "verifier", target).is_err() {
                return Err(Failure("FORBIDDEN", "无本单证据查看权限"));
            }
            let referenced = t["submissions"].as_array().is_some_and(|versions| {
                versions.iter().any(|v| {
                    v["evidence"]["fields"].as_object().is_some_and(|fields| {
                        fields.values().any(|value| {
                            value
                                .as_array()
                                .is_some_and(|files| files.iter().any(|f| f == id))
                        })
                    })
                })
            });
            if !referenced {
                return Err(Failure("FORBIDDEN", "不能查看未提交的草稿附件"));
            }
        }
        let owner = a["owner"].as_str().unwrap_or("").to_owned();
        let result = Repository::file(&mut tx, id, &owner).await?;
        let now = Repository::time(&mut tx).await?;
        state.log(&actor, "evidence_file_view", id, request, &now);
        if let Some(log) = state.audit.last_mut() {
            log["reason"] = json!(reason);
        }
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(result)
    }
}
