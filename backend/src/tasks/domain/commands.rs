use super::*;
use crate::shared::Result;
use crate::tasks::south::port::DomainPort;
use serde_json::Value;

pub trait CommandsRules: DomainPort {
    fn tasks_command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        now: i64,
    ) -> Result<Value> {
        self.tasks_access(actor, action, input)?;
        if action.starts_with("price_") || action.starts_with("template_") {
            return self.rule_command(actor, action, input, generated, now);
        }
        let mut result = if action.starts_with("review_")
            || action == "quality_inspect"
            || action.starts_with("task_repair_")
            || action == "evidence_reveal"
        {
            self.review_command(actor, action, input, generated, now)?
        } else {
            self.execution_command(actor, action, input, generated, now)?
        };
        self.sync_commissions(actor, now)?;
        if action != "evidence_reveal"
            && result["owner"].as_str().is_some_and(|owner| owner != actor)
        {
            result["draft"] = Value::Null;
            result["preview"] = Value::Null;
            result["submissions"] = Value::Null;
            if result["receipt"].is_object() {
                result["receipt"]["evidence"] = Value::Null;
            }
        }
        Ok(result)
    }
}
impl<T: DomainPort + ?Sized> CommandsRules for T {}
