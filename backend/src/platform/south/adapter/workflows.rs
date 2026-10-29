use crate::platform::south::port::State;
use crate::shared::{Result, require};
use serde_json::Value;
pub(crate) fn before(s: &State, actor: &str, action: &str, input: &Value, now: i64) -> Result<()> {
    crate::risk::south::port::rules::check(s, actor, action, now)?;
    if matches!(
        action,
        "settlement_claim"
            | "settlement_handoff"
            | "settlement_handoff_accept"
            | "settlement_register"
            | "settlement_verify"
    ) && let Some(app) = s
        .withdrawals
        .applications
        .get(input["id"].as_str().unwrap_or(""))
    {
        require(
            app["adjustment_review_required"] != true,
            "该申请涉及任务数字调整，须先人工核查，不能继续付款或登记结算",
        )?;
    }
    Ok(())
}
pub(crate) fn checkpoint(s: &State) -> usize {
    s.commissions.entries.len()
}
pub(crate) fn after(
    s: &mut State,
    action: &str,
    result: &Value,
    now: i64,
    checkpoint: usize,
    request: &str,
) -> Result<()> {
    if action == "commission_dispute_resolve" {
        crate::withdrawals::south::port::resume_after_dispute(
            s,
            result["reviewer"].as_str().unwrap_or(""),
            now,
        )?;
    }
    if action == "task_claim" {
        crate::campaigns::south::port::rules::check_claim(s, result)?;
    }
    for entry in s.commissions.entries.iter_mut().skip(checkpoint) {
        entry["request_id"] = serde_json::json!(request);
    }
    crate::risk::south::port::rules::scan(s, now);
    crate::notifications::south::port::rules::sync(s, now);
    Ok(())
}
