use super::validation::{not_found, num};
use crate::community::domain::{Failure, Result, State, invalid};
use serde_json::{Value, json};

impl crate::catalog::south::port::CompletionRecorder for State {
    fn record_completion(
        &mut self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
    ) -> Result<Value> {
        if source.is_empty() || event.is_empty() || source.len() > 128 || event.len() > 128 {
            return Err(invalid("事件来源或事件编号无效"));
        }
        let key = serde_json::to_string(&(source, event)).unwrap();
        if let Some(previous) = self.catalog.completions.get(&key) {
            if previous["claim_id"] != claim || previous["occurred_at"] != occurred_at {
                return Err(Failure("IDEMPOTENCY_CONFLICT", "完成事件内容冲突"));
            }
            return Ok(previous.clone());
        }
        let c = self.catalog.claims.get(claim).ok_or(not_found())?;
        if occurred_at < num(c, "locked_at")? || occurred_at > num(c, "submit_until")? {
            return Err(invalid("完成事件不在任务有效期限内"));
        }
        if c["touch"]["promoter"] == c["owner"] && c["task"]["allow_self"] != true {
            return Err(Failure("SELF_PROMOTION_FORBIDDEN", "自推归因被禁止"));
        }
        let within = c["touch"].is_null()
            || occurred_at.saturating_sub(c["touch"]["time"].as_i64().unwrap_or(0))
                <= num(&c["policy"], "window_seconds")?;
        let value = json!({"claim_id":claim,"source":source,"event_id":event,"occurred_at":occurred_at,"attribution":if within {c["touch"].clone()} else {Value::Null},"status":if within {"attributed"} else {"outside_window"},"commission_created":false});
        // A second source/event cannot create another effective attribution for the same claim.
        if self
            .catalog
            .completions
            .values()
            .any(|v| v["claim_id"] == claim)
        {
            return Err(Failure("ALREADY_COMPLETED", "此任务已有完成归因记录"));
        }
        self.catalog.completions.insert(key, value.clone());
        Ok(value)
    }
}
