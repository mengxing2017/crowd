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
        if action == "task_source_configure" {
            let source = crate::shared::text(input, "source")?;
            crate::shared::text(input, "reason")?;
            crate::shared::require(
                source.len() <= 64
                    && source
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_'),
                "来源编号无效",
            )?;
            let old = self
                .tasks()
                .event_receipts
                .get(&format!("config:{source}"))
                .cloned()
                .unwrap_or(serde_json::json!({"version":0}));
            crate::shared::version(&old, input)?;
            crate::shared::require(input["enabled"].is_boolean(), "须明确来源启停状态")?;
            self.tasks_mut().disabled_sources.retain(|v| v != source);
            if input["enabled"] == false {
                self.tasks_mut().disabled_sources.push(source.into());
            }
            let v = serde_json::json!({"id":format!("config:{source}"),"source":source,"enabled":input["enabled"],"version":old["version"].as_u64().unwrap_or(0)+1,"actor":actor,"at":now});
            self.tasks_mut()
                .event_receipts
                .insert(format!("config:{source}"), v.clone());
            return Ok(v);
        }
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
