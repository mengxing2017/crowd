use crate::shared::{Failure, Result, number, require, text};
use crate::tasks::south::port::{Repository, VerifiedTaskResult};
use serde_json::{Value, json};
#[derive(Clone)]
pub struct ReceiveSignedTaskEvent {
    repo: Repository,
}
impl ReceiveSignedTaskEvent {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
    pub async fn execute(
        &self,
        source: &str,
        signature: &str,
        body: &[u8],
        request: &str,
    ) -> Result<Value> {
        require(body.len() <= 64 * 1024, "任务事件超出处理上限")?;
        let mut tx = self.repo.begin().await?;
        crate::tasks::south::port::verify_signed_event(&mut tx, source, signature, body).await?;
        let input: Value = serde_json::from_slice(body)
            .map_err(|_| Failure("INVALID_INPUT", "任务事件格式错误"))?;
        crate::shared::validate_fields(
            &input,
            &[
                "event_id",
                "claim_id",
                "occurred_at",
                "sent_at",
                "base_cents",
            ],
        )?;
        let payload_hash = crate::tasks::south::port::event_payload_hash(&mut tx, body).await?;
        let base = if input["base_cents"].is_null() {
            None
        } else {
            Some(number(&input, "base_cents")?)
        };
        let received = crate::tasks::south::port::event_received_at(&mut tx).await?;
        let sent = number(&input, "sent_at")?;
        // A source must explicitly publish its transport acceptance window in deployment configuration.
        let window = crate::tasks::south::port::source_window(source)?;
        require(
            sent <= received && received - sent <= window,
            "任务事件传输时间已过期或在未来",
        )?;
        let occurred = number(&input, "occurred_at")?;
        require(occurred <= sent, "成果发生时间不能晚于发送时间")?;
        let event = text(&input, "event_id")?;
        let claim = text(&input, "claim_id")?;
        let mut s = Repository::load(&mut tx).await?;
        require(
            !s.tasks.disabled_sources.iter().any(|v| v == source),
            "任务来源已停用",
        )?;
        require(
            s.tasks
                .instances
                .get(claim)
                .is_some_and(|t| t["template"]["external_source"] == source),
            "来源与任务固定模板不匹配",
        )?;
        let value = s.record_task_result(source, event, claim, occurred, base);
        let now = Repository::time(&mut tx).await?;
        match value {
            Ok(v) => {
                let key = format!("{source}:{event}");
                s.tasks.event_receipts.entry(key.clone()).or_insert(json!({"id":key,"payload_hash":payload_hash,"verification_result":"signature_verified","source":source,"event_id":event,"claim_id":claim,"occurred_at":occurred,"received_at":received,"request_id":request}));
                s.log(source, "signed_task_event", claim, request, &now);
                Repository::save(&mut tx, &s).await?;
                Repository::commit(tx).await?;
                Ok(v)
            }
            Err(e) => {
                if e.0 == "IDEMPOTENCY_CONFLICT" {
                    let key = format!("{source}:{event}:{request}");
                    s.tasks.event_receipts.insert(key.clone(),json!({"id":key,"payload_hash":payload_hash,"verification_result":"signature_verified","source":source,"event_id":event,"claim_id":claim,"occurred_at":occurred,"received_at":received,"status":"conflict_review","request_id":request}));
                }
                s.log_failure(source, "signed_task_event", claim, request, &now, e.0);
                Repository::save(&mut tx, &s).await?;
                Repository::commit(tx).await?;
                Err(e)
            }
        }
    }
}
