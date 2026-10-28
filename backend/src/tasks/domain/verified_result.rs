use crate::shared::Result;
use crate::tasks::south::port::DomainPort;
use crate::tasks::south::port::VerifiedTaskResult;
use serde_json::Value;

impl<T: DomainPort + ?Sized> VerifiedTaskResult for T {
    fn record_task_result(
        &mut self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
        base_cents: Option<i64>,
    ) -> Result<Value> {
        use crate::shared::{Failure, invalid};
        if source.is_empty()
            || source.len() > 128
            || event.is_empty()
            || event.len() > 128
            || base_cents.is_some_and(|n| !(0..=9_007_199_254_740_991).contains(&n))
        {
            return Err(invalid("可信成果参数错误"));
        }
        let t = self
            .tasks()
            .instances
            .get(claim)
            .ok_or(Failure("NOT_FOUND", "任务不存在"))?;
        if occurred_at < t["started_at"].as_i64().ok_or(invalid("任务尚未开始"))?
            || occurred_at > t["execute_until"].as_i64().ok_or(invalid("执行期限缺失"))?
        {
            return Err(invalid("成果发生时间不在执行范围"));
        }
        if t["template"]["external_source"]
            .as_str()
            .is_some_and(|expected| expected != source)
        {
            return Err(Failure("FORBIDDEN", "成果来源与领取时模板不匹配"));
        }
        let value = serde_json::json!({"source":source,"event_id":event,"claim_id":claim,"occurred_at":occurred_at,"base_cents":base_cents});
        if let Some(old) = self.tasks().verified_results.get(claim) {
            if old == &value {
                return Ok(old.clone());
            }
            return Err(Failure("IDEMPOTENCY_CONFLICT", "任务已有不同可信成果"));
        }
        if self
            .tasks()
            .verified_results
            .values()
            .any(|v| v["source"] == source && v["event_id"] == event)
        {
            return Err(Failure("IDEMPOTENCY_CONFLICT", "来源事件已用于其他任务"));
        }
        self.tasks_mut()
            .verified_results
            .insert(claim.into(), value.clone());
        Ok(value)
    }
}
