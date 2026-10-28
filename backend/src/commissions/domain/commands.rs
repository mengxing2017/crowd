use super::{append, cents, number, require, summary};
use crate::{
    platform::domain::State,
    shared::{Failure, Result, text},
};
use serde_json::{Value, json};
pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "commission_adjust_propose"
            | "commission_adjust_review"
            | "commission_dispute"
            | "commission_dispute_resolve"
    )
}
pub fn access(s: &State, actor: &str, a: &str, i: &Value) -> Result<()> {
    let owner = if a == "commission_adjust_review" {
        s.commissions
            .proposals
            .get(text(i, "id")?)
            .ok_or(Failure("NOT_FOUND", "调整单不存在"))?["owner"]
            .as_str()
            .unwrap_or("")
    } else if a == "commission_dispute_resolve" {
        s.commissions
            .disputes
            .get(text(i, "id")?)
            .ok_or(Failure("NOT_FOUND", "争议不存在"))?["owner"]
            .as_str()
            .unwrap_or("")
    } else {
        text(i, "owner")?
    };
    s.authorize(
        actor,
        if a == "commission_adjust_review" || a == "commission_dispute_resolve" {
            "verifier"
        } else {
            "settlement_reviewer"
        },
        owner,
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
    let reason = text(i, "reason")?;
    match a {
        "commission_adjust_propose" => {
            let owner = text(i, "owner")?;
            s.user(owner)?;
            let amount = number(i, "amount_cents")?;
            require(amount != 0, "调整数字不可为零")?;
            let source = text(i, "source")?;
            let business = text(i, "business_key")?;
            require(
                !s.commissions
                    .proposals
                    .values()
                    .any(|p| p["business_key"] == business),
                "同一补记或纠错业务单已经存在",
            )?;
            if let Some(new) = i["new_owner"].as_str() {
                s.user(new)?;
                s.authorize(actor, "settlement_reviewer", new)?;
                require(
                    new != owner && amount < 0,
                    "归属纠错须减少原受益人并指定不同受益人",
                )?;
            }
            let p = json!({"id":id,"owner":owner,"new_owner":i["new_owner"],"amount_cents":amount,"source":source,"business_key":business,"reason":reason,"actor":actor,"at":now,"status":"pending"});
            s.commissions.proposals.insert(id.into(), p.clone());
            Ok(p)
        }
        "commission_adjust_review" => {
            let key = text(i, "id")?;
            let mut p = s
                .commissions
                .proposals
                .get(key)
                .cloned()
                .ok_or(Failure("NOT_FOUND", "调整单不存在"))?;
            require(
                p["status"] == "pending" && p["actor"] != actor,
                "调整须独立复核且不得重复处理",
            )?;
            require(i["approve"].is_boolean(), "请选择复核结论")?;
            if i["approve"] == true {
                let owner = text(&p, "owner")?;
                let amount = number(&p, "amount_cents")?;
                if amount < 0 {
                    let sum = summary(s, owner)?;
                    require(
                        sum["consistent"] == true
                            && sum["available_cents"].as_i64().unwrap_or(0) >= -amount
                            && super::sources(s, owner)?
                                .get(text(&p, "source")?)
                                .copied()
                                .unwrap_or(0)
                                >= -amount,
                        "占用或已结算数字不可直接调减，请先进入争议核查",
                    )?;
                }
                append(
                    s,
                    &format!("adjust:{key}"),
                    owner,
                    if amount > 0 { "increase" } else { "decrease" },
                    amount.abs(),
                    text(&p, "source")?,
                    actor,
                    now,
                    p.clone(),
                )?;
                if let Some(new) = p["new_owner"].as_str() {
                    s.authorize(actor, "verifier", new)?;
                    append(
                        s,
                        &format!("adjust:{key}:recipient"),
                        new,
                        "increase",
                        amount.abs(),
                        key,
                        actor,
                        now,
                        p.clone(),
                    )?;
                }
            }
            p["status"] = json!(if i["approve"] == true {
                "approved"
            } else {
                "rejected"
            });
            p["reviewer"] = json!(actor);
            p["review_reason"] = json!(reason);
            p["reviewed_at"] = json!(now);
            s.commissions.proposals.insert(key.into(), p.clone());
            Ok(p)
        }
        "commission_dispute" => {
            let owner = text(i, "owner")?;
            let n = cents(i, "amount_cents")?;
            let source = text(i, "source")?;
            let sum = summary(s, owner)?;
            require(sum["consistent"] == true, "数字异常，请先核查")?;
            let entitlement: i128 = s
                .commissions
                .entries
                .iter()
                .filter(|e| e["owner"] == owner && e["source"] == source)
                .map(|e| match e["kind"].as_str() {
                    Some("confirmed" | "increase") => {
                        i128::from(e["amount_cents"].as_i64().unwrap_or(0))
                    }
                    Some("decrease") => -i128::from(e["amount_cents"].as_i64().unwrap_or(0)),
                    _ => 0,
                })
                .sum();
            require(
                i128::from(n) <= entitlement
                    && !s.commissions.disputes.values().any(|d| {
                        d["owner"] == owner && d["source"] == source && d["status"] == "open"
                    }),
                "争议范围超出来源数字或该来源已有未完成核查",
            )?;
            let free = super::sources(s, owner)?
                .get(source)
                .copied()
                .unwrap_or(0)
                .max(0);
            let hold = n.min(free);
            let d = json!({"id":id,"owner":owner,"source":source,"amount_cents":hold,"claimed_cents":n,"blocks_applications":n>free,"reason":reason,"responsible":actor,"status":"open","at":now,"version":1});
            append(
                s,
                &format!("dispute:{id}"),
                owner,
                "dispute_hold",
                hold,
                source,
                actor,
                now,
                d.clone(),
            )?;
            s.commissions.disputes.insert(id.into(), d.clone());
            Ok(d)
        }
        "commission_dispute_resolve" => {
            let key = text(i, "id")?;
            let mut d = s
                .commissions
                .disputes
                .get(key)
                .cloned()
                .ok_or(Failure("NOT_FOUND", "争议不存在"))?;
            require(d["status"] == "open", "争议已处理")?;
            require(
                i["confirmed"] == true,
                "请确认核查结论，不代表外部退款或追回",
            )?;
            append(
                s,
                &format!("dispute:{key}:release"),
                text(&d, "owner")?,
                "dispute_release",
                cents(&d, "amount_cents").unwrap_or(0),
                text(&d, "source")?,
                actor,
                now,
                json!({"reason":reason}),
            )?;
            d["status"] = json!("resolved");
            d["reviewer"] = json!(actor);
            d["resolution"] = json!(reason);
            s.commissions.disputes.insert(key.into(), d.clone());
            Ok(d)
        }
        _ => Err(Failure("NOT_FOUND", "佣金操作不存在")),
    }
}
