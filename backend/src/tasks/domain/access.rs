use super::validation::{array, missing, require};
use crate::shared::{Failure, Result, text};
use crate::tasks::south::port::DomainPort;
use serde_json::Value;

pub trait AccessRules: DomainPort {
    fn tasks_access(&self, actor: &str, action: &str, input: &Value) -> Result<()> {
        require(self.user(actor)?.status == "active", "账号当前不可处理任务")?;
        let id = input["id"].as_str().unwrap_or("platform");
        let target = self
            .tasks()
            .instances
            .get(id)
            .and_then(|v| v["product_id"].as_str())
            .unwrap_or("platform");
        match action {
            "price_save" => self.authorize(actor, "rule_manager", "platform"),
            "price_review" => self.authorize(actor, "verifier", "platform"),
            "template_save" | "template_disable" => self.authorize(actor, "operator", "platform"),
            "template_review" => self.authorize(actor, "verifier", "platform"),
            "task_pause" | "task_resume" | "task_expire" | "task_repair_propose" => {
                self.authorize(actor, "operator", target)
            }
            "review_verify" | "task_repair_review" | "review_revoke_review" => {
                self.authorize(actor, "verifier", target)
            }
            "review_claim" | "review_decide" | "review_revoke_propose" | "quality_inspect" => {
                self.authorize(actor, "task_reviewer", target)
            }
            "evidence_reveal" => {
                text(input, "reason")?;
                let t = self.tasks().instances.get(id).ok_or(missing())?;
                let assigned = self.authorize(actor, "task_reviewer", target).is_ok()
                    && t["reviewer"] == actor
                    && matches!(
                        t["status"].as_str(),
                        Some("reviewing" | "awaiting_verification")
                    );
                if !assigned && self.authorize(actor, "verifier", target).is_err() {
                    return Err(Failure("FORBIDDEN", "无本单敏感证据查看权限"));
                }
                Ok(())
            }
            "review_batch_preview" | "review_batch" => {
                for entry in array(input, "entries")? {
                    let task = self
                        .tasks()
                        .instances
                        .get(text(entry, "id")?)
                        .ok_or(missing())?;
                    self.authorize(actor, "task_reviewer", text(task, "product_id")?)?;
                }
                Ok(())
            }
            _ => {
                self.authorize(actor, "user", actor)?;
                require(
                    self.tasks()
                        .instances
                        .get(id)
                        .is_some_and(|v| v["owner"] == actor),
                    "只能处理本人任务",
                )
            }
        }
    }
}
impl<T: DomainPort + ?Sized> AccessRules for T {}
