use crate::{
    commissions::domain::{append, cents, number, require},
    platform::domain::State,
    shared::{Result, text},
};
use serde_json::{Value, json};
fn settle(s: &mut State, app: &mut Value, actor: &str, now: i64) -> Result<()> {
    require(
        !crate::commissions::domain::source_disputed(s, app),
        "本单佣金来源存在争议，须先核查，不得通过审核或结算登记",
    )?;
    let id = text(app, "id")?;
    append(
        s,
        &format!("withdrawal:{id}:settled"),
        text(app, "owner")?,
        "settled",
        cents(app, "amount_cents")?,
        id,
        actor,
        now,
        json!({"registration":app["registration"]["id"]}),
    )?;
    app["status"] = json!("settled");
    Ok(())
}
pub(super) fn register(
    s: &mut State,
    app: &mut Value,
    actor: &str,
    a: &str,
    i: &Value,
    id: &str,
    now: i64,
) -> Result<()> {
    match a {
        "settlement_register" => {
            require(
                app["handler"] == actor
                    && matches!(app["status"].as_str(), Some("processing" | "unknown"))
                    && app["handoff"]["status"] != "pending",
                "须由独占经办登记，交接期间不得登记",
            )?;
            require(
                app["registration"].is_null(),
                "本申请已有登记，请查询原结果或发起关联更正，不得再次付款",
            )?;
            let n = cents(i, "amount_cents")?;
            let paid = number(i, "paid_at")?;
            require(paid > 0 && paid <= now, "线下付款时间不得晚于登记时间")?;
            require(
                i["receipt_id"] == app["receipt_snapshot"]["id"] && i["receipt_confirmed"] == true,
                "必须核对原收款快照，不得改付新资料",
            )?;
            let voucher = text(i, "voucher_file_id")?;
            app["registration"] = json!({"id":id,"actor":actor,"amount_cents":n,"method":app["receipt_snapshot"]["method"],"receipt_id":app["receipt_snapshot"]["id"],"paid_at":paid,"recorded_at":now,"voucher_file_id":voucher,"voucher_owner":actor,"result":"reported_paid","reason":i["reason"],"official_verified":false});
            if crate::commissions::domain::source_disputed(s, app)
                || i["voucher_matches"] != true
                || n != cents(app, "amount_cents")?
            {
                app["status"] = json!("unknown");
                app["uncertainty_reason"] =
                    json!("佣金来源有争议、金额或凭证不符，进入核查并保留占用");
            } else if app["rule"]["registration_verification"] == true {
                app["status"] = json!("awaiting_verification");
            } else {
                settle(s, app, actor, now)?;
            }
        }
        "settlement_verify" => {
            require(
                app["status"] == "awaiting_verification" || app["status"] == "unknown",
                "不在登记复核或核查阶段",
            )?;
            require(
                app["registration"].is_object() && app["registration"]["actor"] != actor,
                "必须由独立人员复核原登记",
            )?;
            require(i["approve"].is_boolean(), "请选择复核结论")?;
            if app["rule"]["separate_roles"] == true {
                require(
                    app["reviewer"] != actor
                        && app["handler"] != actor
                        && app["second_reviewer"] != actor,
                    "结算复核须与审核及经办分离",
                )?;
            }
            if i["approve"] == true {
                require(
                    i["voucher_checked"] == true
                        && app["registration"]["amount_cents"] == app["amount_cents"],
                    "金额或凭证未核对，不可完成登记结算",
                )?;
                settle(s, app, actor, now)?;
            } else {
                app["status"] = json!("unknown");
            }
            app["registration"]["verifier"] = json!(actor);
            app["registration"]["verification_reason"] = i["reason"].clone();
            app["registration"]["verified_at"] = json!(now);
        }
        "settlement_correction_propose" => {
            require(
                app["handler"] == actor && app["registration"].is_object(),
                "仅原经办可申请关联登记更正",
            )?;
            require(app["correction"]["status"] != "pending", "已有更正待复核")?;
            text(i, "evidence")?;
            require(i["confirmed_unpaid"].is_boolean(), "须明确是否确认未付款")?;
            app["correction"] = json!({"id":id,"actor":actor,"status":"pending","registration":app["registration"],"reason":i["reason"],"evidence":i["evidence"],"confirmed_unpaid":i["confirmed_unpaid"],"at":now});
        }
        "settlement_correction_review" => {
            require(
                app["correction"]["status"] == "pending" && app["correction"]["actor"] != actor,
                "更正须独立复核且不可重复",
            )?;
            require(i["approve"].is_boolean(), "请选择复核结论")?;
            // A settled record is never silently reversed or reclaimed. All related evidence remains immutable in events/correction.
            if i["approve"] == true {
                if app["status"] != "settled" && app["correction"]["confirmed_unpaid"] == true {
                    require(
                        i["confirmed_unpaid"] == true && i["processing_stopped"] == true,
                        "须独立确认未付款且处理停止",
                    )?;
                    crate::withdrawals::domain::release(
                        s,
                        app,
                        "rejected",
                        actor,
                        text(i, "reason")?,
                        now,
                    )?;
                } else {
                    app["post_settlement_dispute"] = json!(true);
                }
            }
            app["correction"]["status"] = json!(if i["approve"] == true {
                "approved"
            } else {
                "rejected"
            });
            app["correction"]["reviewer"] = json!(actor);
            app["correction"]["review_reason"] = i["reason"].clone();
            let correction = app["correction"].clone();
            app["events"]
                .as_array_mut()
                .unwrap()
                .push(json!({"action":"registration_correction","record":correction,"at":now}));
        }
        _ => unreachable!(),
    }
    Ok(())
}
