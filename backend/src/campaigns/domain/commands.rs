use crate::campaigns::south::port::DomainPort;
use crate::shared::{Failure, MAX, Result, number, require, text, version};
use serde_json::{Value, json};
pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "campaign_save" | "campaign_transfer" | "campaign_archive"
    )
}
pub fn access<P: DomainPort + ?Sized>(s: &P, a: &str, _action: &str, i: &Value) -> Result<()> {
    s.authorize(a, "operator", text(i, "id")?)
}
pub fn command<P: DomainPort + ?Sized>(
    s: &mut P,
    a: &str,
    action: &str,
    i: &Value,
    _id: &str,
    now: i64,
) -> Result<Value> {
    access(s, a, action, i)?;
    text(i, "reason")?;
    let id = text(i, "id")?;
    let old = s.data().records.get(id).cloned();
    let mut row = old
        .clone()
        .unwrap_or(json!({"id":id,"version":0,"status":"active"}));
    version(&row, i)?;
    require(row["status"] == "active", "归档活动不允许再次修改")?;
    match action {
        "campaign_save" => {
            text(i, "name")?;
            let responsible = text(i, "responsible")?;
            let f = s.facts();
            require(
                f["users"]
                    .as_array()
                    .is_some_and(|u| u.iter().any(|v| v == responsible)),
                "负责人不存在",
            )?;
            require(i["enabled"].is_boolean(), "请选择是否启用计划额度")?;
            require(
                i["products"].as_array().is_some_and(|p| !p.is_empty()),
                "请选择活动产品",
            )?;
            let products = i["products"].as_array().unwrap();
            let mut unique = std::collections::BTreeSet::new();
            for product in products {
                let p = product
                    .as_str()
                    .ok_or(Failure("INVALID_INPUT", "产品编号无效"))?;
                require(
                    unique.insert(p) && f["products"][p].is_object(),
                    "产品不存在或重复",
                )?;
                s.authorize(a, "operator", p)?;
                require(
                    !s.data().records.values().any(|r| {
                        r["id"] != id
                            && r["products"]
                                .as_array()
                                .is_some_and(|ps| ps.contains(product))
                    }),
                    "产品已属于其他活动，不能重置历史占额",
                )?;
            }
            if let Some(ref old) = old {
                require(
                    old["products"] == i["products"],
                    "已有活动不能改换产品集合，避免历史占额转移",
                )?;
            }
            if i["enabled"] == true {
                require(number(i, "limit_cents")? >= 0, "计划额度不能为负")?;
            }
            row["name"] = i["name"].clone();
            row["responsible"] = i["responsible"].clone();
            row["products"] = i["products"].clone();
            row["enabled"] = i["enabled"].clone();
            row["limit_cents"] = if i["enabled"] == true {
                i["limit_cents"].clone()
            } else {
                Value::Null
            };
            let usage = super::queries::report(s, &row)?;
            require(
                i["enabled"] != true
                    || usage["used_cents"].as_i64().unwrap_or(MAX) <= number(i, "limit_cents")?,
                "新上限不能低于现有计划占额及确认数",
            )?;
        }
        "campaign_transfer" => {
            let target = text(i, "target_id")?;
            require(target != id, "请选择不同活动")?;
            s.authorize(a, "operator", target)?;
            let mut other = s
                .data()
                .records
                .get(target)
                .cloned()
                .ok_or(Failure("NOT_FOUND", "目标活动不存在"))?;
            require(
                other["version"] == i["target_version"]
                    && other["status"] == "active"
                    && row["enabled"] == true
                    && other["enabled"] == true,
                "双方活动须开启额度控制且版本有效",
            )?;
            require(i["confirmed"] == true, "请确认仅调整计划数字")?;
            let amount = number(i, "amount_cents")?;
            require(amount > 0, "调整数须为正整数分")?;
            let new = number(&row, "limit_cents")? - amount;
            require(
                new >= super::queries::report(s, &row)?["used_cents"]
                    .as_i64()
                    .unwrap_or(MAX),
                "源活动剩余计划额度不足",
            )?;
            let total = i128::from(number(&other, "limit_cents")?) + i128::from(amount);
            require(total <= i128::from(MAX), "目标额度超出安全整数")?;
            row["limit_cents"] = json!(new);
            other["limit_cents"] = json!(total as i64);
            other["version"] = json!(other["version"].as_u64().unwrap_or(0) + 1);
            s.data_mut().history.push(json!({"action":"campaign_transfer_in","actor":a,"at":now,"source":id,"reason":i["reason"],"amount_cents":amount,"after":other}));
            s.data_mut().records.insert(target.into(), other);
        }
        "campaign_archive" => {
            require(
                old.is_some() && i["confirmed"] == true,
                "请确认存量任务与人工申请的责任人继续处理",
            )?;
            text(&row, "responsible")?;
            row["status"] = json!("archived");
            row["archive_inventory"] = super::queries::report(s, &row)?;
        }
        _ => return Err(Failure("NOT_FOUND", "活动操作不存在")),
    }
    row["version"] = json!(row["version"].as_u64().unwrap_or(0) + 1);
    s.data_mut().history.push(json!({"action":action,"actor":a,"at":now,"reason":i["reason"],"before":old,"after":row,"input":i}));
    s.data_mut().records.insert(id.into(), row.clone());
    Ok(row)
}
/// Executed inside the same locked transaction as a new claim; failure rolls the claim back.
pub fn check_claim<P: DomainPort + ?Sized>(s: &P, result: &Value) -> Result<()> {
    for campaign in s.data().records.values().filter(|r| {
        r["products"]
            .as_array()
            .is_some_and(|ps| ps.contains(&result["product_id"]))
    }) {
        require(campaign["status"] == "active", "活动已归档，停止新增领取")?;
        if campaign["enabled"] == true {
            require(
                super::queries::report(s, campaign)?["used_cents"]
                    .as_i64()
                    .unwrap_or(MAX)
                    <= number(campaign, "limit_cents")?,
                "活动计划数字额度不足，不代表真实资金不足",
            )?;
        }
    }
    Ok(())
}
