use super::{active_rule, rule_command};
use crate::withdrawals::south::port::DomainPort;
use crate::{
    shared::{Failure, Result, text},
    shared::{cents, number, require, version},
};
use serde_json::{Value, json};
pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "withdrawal_rule_propose"
            | "withdrawal_rule_review"
            | "withdrawal_preview"
            | "withdrawal_apply"
            | "withdrawal_cancel"
            | "withdrawal_receipt_reveal"
    )
}
pub fn access<P: DomainPort + ?Sized>(s: &P, actor: &str, a: &str, i: &Value) -> Result<()> {
    require(s.user(actor)?.status == "active", "账号不可执行提现操作")?;
    match a {
        "withdrawal_rule_propose" => s.authorize(actor, "rule_manager", "platform"),
        "withdrawal_rule_review" => s.authorize(actor, "verifier", "platform"),
        "withdrawal_cancel" | "withdrawal_receipt_reveal" => {
            let app = s
                .withdrawals()
                .applications
                .get(text(i, "id")?)
                .ok_or(Failure("NOT_FOUND", "申请不存在"))?;
            if app["owner"] == actor {
                return Ok(());
            }
            if a == "withdrawal_cancel" {
                return Err(Failure("FORBIDDEN", "只能撤销本人申请"));
            }
            let owner = text(app, "owner")?;
            if s.authorize(actor, "settlement_reviewer", owner).is_err()
                && (app["handler"] != actor || s.authorize(actor, "manual_handler", owner).is_err())
            {
                s.authorize(actor, "verifier", owner)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
pub fn release<P: DomainPort + ?Sized>(
    s: &mut P,
    app: &mut Value,
    status: &str,
    actor: &str,
    reason: &str,
    now: i64,
) -> Result<()> {
    require(
        !matches!(
            app["status"].as_str(),
            Some("cancelled" | "rejected" | "settled")
        ),
        "此申请已经结束，不得重复释放",
    )?;
    let id = text(app, "id")?;
    s.ledger_append(
        &format!("withdrawal:{id}:release"),
        text(app, "owner")?,
        "release",
        cents(app, "amount_cents")?,
        id,
        actor,
        now,
        json!({"reason":reason,"sources":s.commission_hold_sources(id)}),
    )?;
    app["status"] = json!(status);
    s.sync_commissions(actor, now)?;
    Ok(())
}
fn snapshot<P: DomainPort + ?Sized>(s: &P, actor: &str, i: &Value, r: &Value) -> Result<Value> {
    let u = s.user(actor)?;
    require(
        !u.restrictions
            .iter()
            .any(|x| x["active"] != false && (x["scope"] == "withdrawal" || x["scope"] == "all")),
        "账号申请功能受限，请联系客服",
    )?;
    let rid = text(i, "receipt_id")?;
    let receipt = u
        .receipts
        .iter()
        .find(|v| v["id"] == rid && v["method"].is_string())
        .ok_or(Failure("RECEIPT_INVALID", "收款资料不存在"))?;
    require(
        !u.receipts.iter().any(|v| v["disables"] == rid),
        "收款资料已停用",
    )?;
    require(
        i["receipt_version"] == receipt["version"],
        "收款资料版本变化，请重新确认",
    )?;
    require(
        r["methods"]
            .as_array()
            .is_some_and(|m| m.contains(&receipt["method"])),
        "该收款方式未启用",
    )?;
    if receipt["method"] == "wechat" {
        text(receipt, "file_id")?;
        require(receipt["status"] == "confirmed", "请确认本人收款二维码")?;
    } else {
        require(
            receipt["status"] == "confirmed",
            "请按已发布政策重新确认支付宝收款资料，草稿不可申请",
        )?;
        s.validate_alipay(receipt, r)?;
        require(
            r["alipay_policy_confirmed"] == true,
            "支付宝资料检查政策尚未确认",
        )?;
    }
    let n = cents(i, "amount_cents")?;
    require(
        n >= cents(r, "min_cents")? && n <= cents(r, "max_cents")?,
        "申请不符合当前最低或最高限额，请查看规则",
    )?;
    let sum = s.commission_summary(actor)?;
    require(
        sum["consistent"] == true,
        "数字记录异常，暂停新增申请并联系核查",
    )?;
    if i["summary_version"] != sum["version"] {
        return Err(Failure(
            "VERSION_CONFLICT",
            "数字汇总版本变化，请查询最新数字及在途申请",
        ));
    }
    if n > number(&sum, "available_cents")? {
        return Err(Failure(
            "INSUFFICIENT_AVAILABLE",
            "可申请数字不足，请查询最新汇总及在途申请；不会部分申请",
        ));
    }
    Ok(receipt.clone())
}
pub fn command<P: DomainPort + ?Sized>(
    s: &mut P,
    actor: &str,
    a: &str,
    i: &Value,
    id: &str,
    now: i64,
) -> Result<Value> {
    access(s, actor, a, i)?;
    if a.starts_with("withdrawal_rule_") {
        return rule_command(s, actor, a, i, id, now);
    }
    if a == "withdrawal_preview" || a == "withdrawal_apply" {
        let r = active_rule(s)?;
        let receipt = snapshot(s, actor, i, &r)?;
        let n = cents(i, "amount_cents")?;
        let apps: Vec<_> = s
            .withdrawals()
            .applications
            .values()
            .filter(|v| v["owner"] == actor)
            .collect();
        // Calendar day is explicitly UTC+08:00; cancelled/rejected attempts still count against configured submission limits.
        let today: Vec<_> = apps
            .iter()
            .filter(|v| {
                (v["created_at"].as_i64().unwrap_or(0) + 28800) / 86400 == (now + 28800) / 86400
            })
            .collect();
        require(
            (today.len() as i64) < cents(&r, "daily_count")?,
            "已达当日申请次数规则",
        )?;
        let total: i128 = apps
            .iter()
            .map(|v| i128::from(v["amount_cents"].as_i64().unwrap_or(0)))
            .sum();
        let day: i128 = today
            .iter()
            .map(|v| i128::from(v["amount_cents"].as_i64().unwrap_or(0)))
            .sum();
        require(
            day + i128::from(n) <= i128::from(cents(&r, "daily_cents")?)
                && total + i128::from(n) <= i128::from(cents(&r, "lifetime_cents")?),
            "已达日累计或累计申请限额",
        )?;
        if a == "withdrawal_preview" {
            let p = json!({"id":id,"owner":actor,"amount_cents":n,"receipt_id":i["receipt_id"],"receipt_version":i["receipt_version"],"summary_version":i["summary_version"],"rule_id":r["id"],"at":now,"notice":"无额外扣减；付款由人员在系统外执行，不承诺自动到账"});
            s.withdrawals_mut().previews.insert(id.into(), p.clone());
            return Ok(p);
        }
        let p = s
            .withdrawals()
            .previews
            .get(text(i, "preview_id")?)
            .ok_or(Failure("PREVIEW_REQUIRED", "请先预览确认申请"))?;
        require(
            p["owner"] == actor
                && p["rule_id"] == r["id"]
                && p["amount_cents"] == n
                && p["receipt_id"] == i["receipt_id"]
                && p["receipt_version"] == i["receipt_version"]
                && p["summary_version"] == i["summary_version"]
                && i["confirmed"] == true,
            "请重新预览并确认当前金额、规则和资料版本",
        )?;
        require(
            !s.withdrawals()
                .applications
                .values()
                .any(|a| a["preview_id"] == i["preview_id"]),
            "该预览已生成申请，请查询原结果",
        )?;
        let app = json!({"id":id,"owner":actor,"version":1,"status":"pending_review","amount_cents":n,"currency":"CNY","rule":r,"receipt_snapshot":receipt,"receipt_confirmed_at":now,"created_at":now,"updated_at":now,"preview_id":i["preview_id"],"events":[{"action":"apply","actor":actor,"at":now}],"handler":null,"registration":null});
        let allocations = s.allocate_commissions(actor, n)?;
        s.ledger_append(
            &format!("withdrawal:{id}:hold"),
            actor,
            "hold",
            n,
            id,
            actor,
            now,
            json!({"rule_id":r["id"],"receipt_id":i["receipt_id"],"sources":allocations}),
        )?;
        s.withdrawals_mut()
            .applications
            .insert(id.into(), app.clone());
        return Ok(super::queries::mask(app));
    }
    let key = text(i, "id")?;
    let mut app = s
        .withdrawals()
        .applications
        .get(key)
        .cloned()
        .ok_or(Failure("NOT_FOUND", "申请不存在"))?;
    if a == "withdrawal_receipt_reveal" {
        text(i, "reason")?;
        return Ok(
            json!({"id":key,"receipt_snapshot":app["receipt_snapshot"],"registration":app["registration"],"official_verified":false}),
        );
    }
    version(&app, i)?;
    require(
        matches!(
            app["status"].as_str(),
            Some("pending_review" | "awaiting_second_review" | "ready")
        ) && app["handler"].is_null(),
        "已进入人工处理或结束，必须先确认停止处理且未付款",
    )?;
    let reason = text(i, "reason")?;
    release(s, &mut app, "cancelled", actor, reason, now)?;
    app["version"] = json!(app["version"].as_u64().unwrap_or(0) + 1);
    app["updated_at"] = json!(now);
    app["events"]
        .as_array_mut()
        .unwrap()
        .push(json!({"action":a,"actor":actor,"at":now,"reason":reason}));
    s.withdrawals_mut()
        .applications
        .insert(key.into(), app.clone());
    Ok(super::queries::mask(app))
}
