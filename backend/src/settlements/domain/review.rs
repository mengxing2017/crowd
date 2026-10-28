use crate::settlements::south::port::DomainPort;
use crate::{
    shared::{Failure, Result, text},
    shared::{cents, require},
};
use serde_json::{Value, json};
pub(super) fn review<P: DomainPort + ?Sized>(
    s: &mut P,
    app: &mut Value,
    actor: &str,
    a: &str,
    i: &Value,
    now: i64,
) -> Result<()> {
    require(
        app["status"]
            == if a == "settlement_review" {
                "pending_review"
            } else {
                "awaiting_second_review"
            },
        "申请不在对应待审阶段",
    )?;
    if a == "settlement_second_review" {
        require(app["reviewer"] != actor, "高额复核须由不同身份处理")?;
    }
    require(i["approve"].is_boolean(), "请选择审核结论")?;
    if i["approve"] != true {
        s.release_withdrawal(app, "rejected", actor, text(i, "reason")?, now)?;
        return Ok(());
    }
    require(
        !s.commission_source_disputed(app),
        "本单佣金来源存在争议，须先核查，不得通过审核或结算登记",
    )?;
    require(
        i["receipt_checked"] == true && i["source_checked"] == true,
        "必须人工核对收款快照及佣金来源；不代表官方收款验证",
    )?;
    let owner = text(app, "owner")?;
    let id = text(app, "id")?;
    let sum = s.commission_summary(owner)?;
    require(
        sum["consistent"] == true
            && s.commissions_entries().iter().any(|e| {
                e["kind"] == "hold"
                    && e["source"] == id
                    && e["owner"] == owner
                    && e["amount_cents"] == app["amount_cents"]
            }),
        "申请缺少有效数字来源或占用，请核查",
    )?;
    if a == "settlement_review" {
        app["reviewer"] = json!(actor);
        let threshold = app["rule"]["second_review_cents"].as_i64().unwrap_or(0);
        app["status"] = json!(
            if threshold > 0 && cents(app, "amount_cents")? >= threshold {
                "awaiting_second_review"
            } else {
                "ready"
            }
        );
    } else {
        app["second_reviewer"] = json!(actor);
        app["status"] = json!("ready");
    }
    Ok(())
}
pub(super) fn batch<P: DomainPort + ?Sized>(
    s: &mut P,
    actor: &str,
    a: &str,
    i: &Value,
    id: &str,
    now: i64,
) -> Result<Value> {
    let entries = i["entries"]
        .as_array()
        .ok_or(Failure("INVALID_INPUT", "请选择批次条目"))?;
    require(
        !entries.is_empty() && entries.len() <= 50,
        "批次限制为1至50条",
    )?;
    let mut seen = std::collections::BTreeSet::new();
    let mut total = 0_i128;
    let mut results = vec![];
    for e in entries {
        let key = text(e, "id")?;
        require(seen.insert(key), "批次条目重复")?;
        let app = s.withdrawals_applications().get(key);
        let valid = app.is_some_and(|v| {
            v["status"] == "pending_review"
                && v["version"] == e["version"]
                && v["owner"] != actor
                && s.authorize(
                    actor,
                    "settlement_reviewer",
                    v["owner"].as_str().unwrap_or(""),
                )
                .is_ok()
                && v["receipt_snapshot"]["method"].is_string()
        });
        if valid {
            total += i128::from(app.unwrap()["amount_cents"].as_i64().unwrap_or(0));
        }
        results.push(json!({"id":key,"eligible":valid,"issue":if valid{""}else{"资料不全、权限不足、版本改变或已处理"}}));
    }
    require(total <= 9_007_199_254_740_991, "批次总额超出安全范围")?;
    if a == "settlement_batch_preview" {
        let p = json!({"id":id,"kind":"settlement_batch","actor":actor,"entries":entries,"results":results,"count":entries.len(),"amount_cents":total as i64,"at":now,"notice":"仅审核清单，不是转账文件"});
        s.save_withdrawal_preview(id, p.clone());
        return Ok(p);
    }
    let p = s
        .withdrawals_previews()
        .get(text(i, "preview_id")?)
        .ok_or(Failure("PREVIEW_REQUIRED", "请先预览批次"))?;
    require(
        p["kind"] == "settlement_batch"
            && p["actor"] == actor
            && p["entries"] == i["entries"]
            && i["confirmed"] == true
            && results.iter().all(|r| r["eligible"] == true),
        "批次已变化或含异常项，请重新预览",
    )?;
    let mut outcomes = vec![];
    for e in entries {
        let result = super::dispatch::command(
            s,
            actor,
            "settlement_review",
            &json!({"id":e["id"],"version":e["version"],"approve":i["approve"],"reason":text(i,"reason")?,"receipt_checked":i["receipt_checked"],"source_checked":i["source_checked"]}),
            id,
            now,
        )?;
        outcomes.push(json!({"id":result["id"],"status":result["status"]}));
    }
    Ok(json!({"count":outcomes.len(),"results":outcomes,"amount_cents":total as i64}))
}
