use crate::platform::south::port::State;
use crate::shared::{Failure, Result};
use serde_json::json;
pub(crate) fn pause_for_adjustment(s: &mut State, id: &str, a: &str, now: i64) -> Result<()> {
    let app = s
        .withdrawals
        .applications
        .get_mut(id)
        .ok_or(Failure("NOT_FOUND", "申请不存在"))?;
    if !matches!(
        app["status"].as_str(),
        Some("settled" | "cancelled" | "rejected")
    ) {
        app["adjustment_review_required"] = json!(true);
        app["adjustment_reason"] =
            json!("任务数字调整待核查；保留原占用、资料与经办，不得继续付款");
        app["version"] = json!(app["version"].as_u64().unwrap_or(0) + 1);
        app["events"]
            .as_array_mut()
            .ok_or(Failure("SERVICE_UNAVAILABLE", "申请历史不可读取"))?
            .push(json!({"action":"adjustment_pause","actor":a,"at":now}));
    }
    Ok(())
}

/// Only a verified dispute conclusion may resume a paused application. Receipt and handler stay fixed.
pub(crate) fn resume_after_dispute(s: &mut State, actor: &str, now: i64) -> Result<()> {
    let ids: Vec<String> = s
        .withdrawals
        .applications
        .values()
        .filter(|a| {
            a["adjustment_review_required"] == true
                && s.authorize(actor, "verifier", a["owner"].as_str().unwrap_or(""))
                    .is_ok()
                && !matches!(
                    a["status"].as_str(),
                    Some("cancelled" | "rejected" | "settled")
                )
                && !crate::commissions::south::port::rules::source_disputed(s, a)
        })
        .filter_map(|a| a["id"].as_str().map(String::from))
        .collect();
    for id in ids {
        let app = s
            .withdrawals
            .applications
            .get_mut(&id)
            .ok_or(Failure("NOT_FOUND", "申请不存在"))?;
        app["adjustment_review_required"] = json!(false);
        app["version"] = json!(app["version"].as_u64().unwrap_or(0) + 1);
        app["updated_at"] = json!(now);
        app["events"].as_array_mut().ok_or(Failure("SERVICE_UNAVAILABLE","申请历史不可读取"))?.push(json!({"action":"adjustment_dispute_reviewed","actor":actor,"at":now,"notice":"依据数字复核结论恢复原人工处理流程，不代表实际付款成功"}));
    }
    Ok(())
}
