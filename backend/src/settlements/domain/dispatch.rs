use crate::{
    commissions::domain::{require, version},
    platform::domain::State,
    shared::{Failure, Result, text},
};
use serde_json::{Value, json};
pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "settlement_review"
            | "settlement_second_review"
            | "settlement_batch_preview"
            | "settlement_batch_review"
            | "settlement_claim"
            | "settlement_handoff"
            | "settlement_handoff_accept"
            | "settlement_exception"
            | "settlement_unknown"
            | "settlement_confirm_unpaid"
            | "settlement_register"
            | "settlement_verify"
            | "settlement_correction_propose"
            | "settlement_correction_review"
    )
}
pub fn access(s: &State, actor: &str, a: &str, i: &Value) -> Result<()> {
    require(s.user(actor)?.status == "active", "账号不可处理结算")?;
    if a.starts_with("settlement_batch_") {
        s.authorize(actor, "settlement_reviewer", "platform")?;
        return Ok(());
    }
    let app = s
        .withdrawals
        .applications
        .get(text(i, "id")?)
        .ok_or(Failure("NOT_FOUND", "申请不存在"))?;
    s.authorize(
        actor,
        if matches!(
            a,
            "settlement_second_review" | "settlement_verify" | "settlement_correction_review"
        ) {
            "verifier"
        } else if a == "settlement_review" {
            "settlement_reviewer"
        } else {
            "manual_handler"
        },
        text(app, "owner")?,
    )
}
pub fn command(
    s: &mut State,
    actor: &str,
    a: &str,
    i: &Value,
    id: &str,
    now: i64,
) -> Result<Value> {
    access(s, actor, a, i)?;
    if a.starts_with("settlement_batch_") {
        return super::review::batch(s, actor, a, i, id, now);
    }
    let key = text(i, "id")?;
    let mut app = s
        .withdrawals
        .applications
        .get(key)
        .cloned()
        .ok_or(Failure("NOT_FOUND", "申请不存在"))?;
    version(&app, i)?;
    text(i, "reason")?;
    require(app["owner"] != actor, "不得审核、经办或复核本人申请")?;
    match a {
        "settlement_review" | "settlement_second_review" => {
            super::review::review(s, &mut app, actor, a, i, now)?
        }
        "settlement_claim"
        | "settlement_handoff"
        | "settlement_handoff_accept"
        | "settlement_exception"
        | "settlement_unknown"
        | "settlement_confirm_unpaid" => super::handling::handle(s, &mut app, actor, a, i, now)?,
        _ => super::registration::register(s, &mut app, actor, a, i, id, now)?,
    }
    app["version"] = json!(app["version"].as_u64().unwrap_or(0) + 1);
    app["updated_at"] = json!(now);
    let event_version = app["version"].clone();
    let exception = app["exception"].clone();
    app["events"].as_array_mut().unwrap().push(
        json!({"action":a,"actor":actor,"at":now,"reason":i["reason"],"version":event_version,"exception":exception}),
    );
    s.withdrawals.applications.insert(key.into(), app.clone());
    crate::withdrawals::domain::read(s, actor, "withdrawal_detail", key, now)
}
