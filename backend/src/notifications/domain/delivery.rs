use crate::notifications::south::port::{DeliveryOutcome, DomainPort};
use crate::shared::{Failure, Result, number, require, text};
use serde_json::{Value, json};
pub(crate) fn reserve<P: DomainPort + ?Sized>(
    s: &mut P,
    id: &str,
    attempt: &str,
    now: i64,
) -> Result<Value> {
    let mut m = s
        .data()
        .records
        .get(id)
        .cloned()
        .ok_or(Failure("NOT_FOUND", "消息不存在"))?;
    require(
        m["kind"] == "message"
            && m["external_status"] != "delivered"
            && m["external_status"] != "permanent_failure"
            && m["external_status"] != "no_consent",
        "消息不在可发送状态",
    )?;
    let template = m["template_snapshot"].clone();
    let template_id = text(&m, "template_id")?;
    let consent_key = format!("consent:{}:{template_id}", text(&m, "owner")?);
    require(
        s.data()
            .records
            .get(&consent_key)
            .is_some_and(|c| c["granted"] == true),
        "用户未主动授权此模板，保留站内消息",
    )?;
    let max = number(&template, "max_attempts")?;
    let retry = number(&template, "retry_seconds")?;
    require(max > 0 && retry > 0, "外部通知必须显式配置有限重试策略")?;
    let n = m["attempts"].as_i64().unwrap_or(0);
    require(n < max, "已达到通知重试上限")?;
    require(
        m["next_attempt_at"].as_i64().unwrap_or(0) <= now,
        "尚未到达通知重试时间",
    )?;
    let next = now
        .checked_add(retry)
        .filter(|t| *t <= crate::shared::MAX)
        .ok_or(Failure("INVALID_INPUT", "通知重试时间超出安全范围"))?;
    m["attempts"] = json!(n + 1);
    m["attempt_key"] = json!(attempt);
    m["next_attempt_at"] = json!(next);
    m["external_status"] = json!("sending");
    s.data_mut()
        .history
        .push(json!({"id":id,"attempt":attempt,"at":now,"status":"sending"}));
    s.data_mut().records.insert(id.into(), m.clone());
    Ok(m)
}
pub(crate) fn finish<P: DomainPort + ?Sized>(
    s: &mut P,
    id: &str,
    attempt: &str,
    outcome: DeliveryOutcome,
    now: i64,
) -> Result<Value> {
    let mut m = s
        .data()
        .records
        .get(id)
        .cloned()
        .ok_or(Failure("NOT_FOUND", "消息不存在"))?;
    require(
        m["attempt_key"] == attempt && m["external_status"] == "sending",
        "通知发送尝试已被更新或已记录结果",
    )?;
    m["external_status"] = json!(match outcome {
        DeliveryOutcome::Delivered => "delivered",
        DeliveryOutcome::TemporaryFailure =>
            if m["attempts"].as_i64().unwrap_or(0)
                >= m["template_snapshot"]["max_attempts"].as_i64().unwrap_or(0)
            {
                "retry_exhausted"
            } else {
                "temporary_failure"
            },
        DeliveryOutcome::PermanentFailure => "permanent_failure",
        DeliveryOutcome::NoConsent => "no_consent",
    });
    s.data_mut()
        .history
        .push(json!({"id":id,"attempt":attempt,"at":now,"status":m["external_status"]}));
    s.data_mut().records.insert(id.into(), m.clone());
    Ok(m)
}
