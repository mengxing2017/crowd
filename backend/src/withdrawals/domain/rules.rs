use crate::{
    commissions::domain::{cents, number, require},
    platform::domain::State,
    shared::{Failure, Result, text},
};
use serde_json::{Value, json};
pub fn active_rule(s: &State) -> Result<Value> {
    s.withdrawals
        .rules
        .iter()
        .rev()
        .find(|r| r["status"] == "published")
        .cloned()
        .ok_or(Failure(
            "RULE_NOT_CONFIGURED",
            "提现规则尚未发布，暂不可申请",
        ))
}
pub fn rule_command(
    s: &mut State,
    actor: &str,
    a: &str,
    i: &Value,
    id: &str,
    now: i64,
) -> Result<Value> {
    s.authorize(
        actor,
        if a == "withdrawal_rule_review" {
            "verifier"
        } else {
            "rule_manager"
        },
        "platform",
    )?;
    text(i, "reason")?;
    if a == "withdrawal_rule_propose" {
        let min = cents(i, "min_cents")?;
        let max = cents(i, "max_cents")?;
        require(max >= min, "最高申请值不得低于最低值")?;
        cents(i, "daily_count")?;
        cents(i, "daily_cents")?;
        cents(i, "lifetime_cents")?;
        for key in ["second_review_cents", "reminder_seconds"] {
            require(number(i, key)? >= 0, "可选阈值须为非负整数，零表示不启用")?;
        }
        for key in ["registration_verification", "separate_roles"] {
            require(i[key].is_boolean(), "必须明确复核与岗位分离配置")?;
        }
        let methods = i["methods"]
            .as_array()
            .ok_or(Failure("INVALID_INPUT", "请选择收款方式"))?;
        require(
            !methods.is_empty() && methods.iter().all(|m| m == "wechat" || m == "alipay"),
            "收款方式无效",
        )?;
        if methods.iter().any(|m| m == "alipay") {
            text(i, "alipay_policy")?;
            require(
                i["alipay_policy_confirmed"] == true,
                "启用支付宝必须明确平台收款资料检查政策，不代表官方验证",
            )?;
        }
        let mut r = i.clone();
        r["id"] = json!(id);
        r["version"] = json!(s.withdrawals.rules.len() + 1);
        r["status"] = json!("draft");
        r["actor"] = json!(actor);
        r["at"] = json!(now);
        r["currency"] = json!("CNY");
        r["extra_deduction_cents"] = json!(0);
        s.withdrawals.rules.push(r.clone());
        Ok(r)
    } else {
        let key = text(i, "id")?;
        let r = s
            .withdrawals
            .rules
            .iter()
            .find(|r| r["id"] == key)
            .cloned()
            .ok_or(Failure("NOT_FOUND", "规则不存在"))?;
        require(
            r["status"] == "draft" && r["actor"] != actor,
            "规则须独立复核且只处理草稿",
        )?;
        require(i["approve"].is_boolean(), "请选择复核结论")?;
        if i["approve"] == true {
            for old in &mut s.withdrawals.rules {
                if old["status"] == "published" {
                    old["status"] = json!("superseded");
                }
            }
        }
        let r = s
            .withdrawals
            .rules
            .iter_mut()
            .find(|r| r["id"] == key)
            .unwrap();
        r["status"] = json!(if i["approve"] == true {
            "published"
        } else {
            "rejected"
        });
        r["reviewer"] = json!(actor);
        r["review_reason"] = i["reason"].clone();
        r["reviewed_at"] = json!(now);
        Ok(r.clone())
    }
}
