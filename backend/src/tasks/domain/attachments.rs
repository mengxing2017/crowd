use crate::platform::domain::State;
use crate::shared::{Failure, Result};
use serde_json::json;
pub(crate) fn validate_upload(
    state: &State,
    actor: &str,
    purpose: &str,
    size: usize,
) -> Result<()> {
    let task_id = purpose.strip_prefix("task_evidence:");
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
    Ok(())
}
pub(crate) fn register_upload(
    state: &mut State,
    actor: &str,
    task: Option<&str>,
    id: &str,
    size: usize,
) {
    if let Some(task) = task {
        state
            .tasks
            .attachments
            .insert(id.into(), json!({"task_id":task,"owner":actor,"size":size}));
    }
}
