use crate::{
    commissions::domain::require,
    platform::domain::State,
    shared::{Result, text},
};
use serde_json::{Value, json};
pub(super) fn handle(
    s: &mut State,
    app: &mut Value,
    actor: &str,
    a: &str,
    i: &Value,
    now: i64,
) -> Result<()> {
    if a == "settlement_claim" {
        require(
            !crate::commissions::domain::source_disputed(s, app),
            "本单佣金来源有争议，先核查，不可开始线下处理",
        )?;
        require(
            app["status"] == "ready" && app["handler"].is_null(),
            "该申请未完成审核或已被领取",
        )?;
        if app["rule"]["separate_roles"] == true {
            require(
                app["reviewer"] != actor && app["second_reviewer"] != actor,
                "经办须与审核、复核人员分离",
            )?;
        }
        app["handler"] = json!(actor);
        app["claimed_at"] = json!(now);
        app["status"] = json!("processing");
        return Ok(());
    }
    if a == "settlement_handoff_accept" {
        require(
            app["handoff"]["to"] == actor && app["handoff"]["status"] == "pending",
            "仅交接接收人可确认",
        )?;
        require(
            app["status"] == "processing" || app["status"] == "unknown",
            "此阶段不能交接",
        )?;
        if app["rule"]["separate_roles"] == true {
            require(
                app["reviewer"] != actor && app["second_reviewer"] != actor,
                "接收经办须与审核人员分离",
            )?;
        }
        app["handler"] = json!(actor);
        app["handoff"]["status"] = json!("accepted");
        app["handoff"]["accepted_at"] = json!(now);
        return Ok(());
    }
    require(app["handler"] == actor, "只允许本单独占经办处理")?;
    require(
        matches!(app["status"].as_str(), Some("processing" | "unknown")),
        "当前状态不可变更人工处理结果",
    )?;
    match a {
        "settlement_handoff" => {
            require(app["handoff"]["status"] != "pending", "已有交接待接收")?;
            let to = text(i, "to")?;
            require(to != actor && app["owner"] != to, "须选择其他授权经办人员")?;
            s.authorize(to, "manual_handler", text(app, "owner")?)?;
            let paid = text(i, "paid_status")?;
            require(
                ["unpaid", "paid", "unknown"].contains(&paid),
                "须说明是否已线下付款",
            )?;
            app["handoff"] = json!({"from":actor,"to":to,"at":now,"reason":i["reason"],"paid_status":paid,"status":"pending"});
            if paid != "unpaid" {
                app["status"] = json!("unknown");
            }
        }
        "settlement_exception" => {
            let kind = text(i, "kind")?;
            require(
                ["receipt_error", "unpaid", "unknown", "registration_error"].contains(&kind),
                "异常类型无效",
            )?;
            if matches!(
                app["exception"]["kind"].as_str(),
                Some("receipt_error" | "registration_error")
            ) {
                require(
                    app["exception"]["kind"] == kind,
                    "资料或登记错误不能降级为其他异常以绕过核查",
                )?;
            }
            let assignee = text(i, "assignee")?;
            require(app["owner"] != assignee, "不可指派申请人处理本人异常")?;
            s.authorize(assignee, "manual_handler", text(app, "owner")?)?;
            require(
                app["handoff"]["status"] != "pending",
                "交接期间须先完成交接",
            )?;
            app["exception"] = json!({"kind":kind,"assignee":assignee,"reported_by":actor,"reason":i["reason"],"at":now});
            app["status"] = json!("unknown");
            app["uncertainty_reason"] = i["reason"].clone();
        }
        "settlement_unknown" => {
            app["status"] = json!("unknown");
            app["uncertainty_reason"] = i["reason"].clone();
        }
        "settlement_confirm_unpaid" => {
            require(
                i["confirmed_unpaid"] == true
                    && i["processing_stopped"] == true
                    && app["handoff"]["status"] != "pending",
                "必须确认未付款且已停止处理，交接须先完成",
            )?;
            require(
                app["registration"].is_null(),
                "已有付款登记须先独立核查，不能直接释放",
            )?;
            crate::withdrawals::domain::release(
                s,
                app,
                "rejected",
                actor,
                text(i, "reason")?,
                now,
            )?;
        }
        _ => unreachable!(),
    }
    Ok(())
}
