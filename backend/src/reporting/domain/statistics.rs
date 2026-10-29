use crate::reporting::south::port::DomainPort;
use crate::shared::{Result, number, require};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn allowed<P: DomainPort + ?Sized>(s: &P, a: &str, t: &str) -> bool {
    s.authorize(a, "auditor", t).is_ok() || s.authorize(a, "operator", t).is_ok()
}
pub(super) fn owner_allowed<P: DomainPort + ?Sized>(s: &P, a: &str, t: &str) -> bool {
    s.authorize(a, "auditor", t).is_ok()
}
fn at(v: &Value, key: &str) -> Option<i64> {
    v[key].as_i64()
}
pub(super) fn snapshot<P: DomainPort + ?Sized>(
    s: &P,
    a: &str,
    i: &Value,
    now: i64,
) -> Result<Value> {
    let from = number(i, "from")?;
    let until = number(i, "until")?;
    require(
        from >= 0 && until >= from && until <= now,
        "统计时间范围错误或超出当前水位",
    )?;
    let product = i["product_id"].as_str().unwrap_or("");
    let source = i["source"].as_str().unwrap_or("");
    let f = s.facts();
    let in_range = |t: Option<i64>| t.is_some_and(|t| t >= from && t <= until);
    let claims: Vec<Value> = f["claims"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.values())
        .filter(|c| {
            allowed(s, a, c["product_id"].as_str().unwrap_or(""))
                && (product.is_empty() || c["product_id"] == product)
                && (source.is_empty()
                    || c["promoter"] == source
                    || c["touch"]["promoter"] == source)
                && in_range(at(c, "created_at").or_else(|| at(c, "locked_at")))
        })
        .cloned()
        .collect();
    let ids: BTreeSet<&str> = claims.iter().filter_map(|c| c["id"].as_str()).collect();
    let mut products: BTreeMap<String, Value> = BTreeMap::new();
    let mut submitted = 0;
    let mut approved = 0;
    let mut rejected = 0;
    let mut supplemented = 0;
    let mut durations = vec![];
    for c in &claims {
        let p = c["product_id"].as_str().unwrap_or("");
        let r=products.entry(p.into()).or_insert(json!({"product_id":p,"claims":0,"planned_cents":0,"confirmed_cents":0,"adjusted_cents":0,"settled_cents":0,"entry_ids":[]}));
        r["claims"] = json!(r["claims"].as_i64().unwrap_or(0) + 1);
        r["planned_cents"] = json!(
            r["planned_cents"].as_i64().unwrap_or(0)
                + c["reserved_cents"]
                    .as_i64()
                    .or_else(|| c["task"]["commission_cents"].as_i64())
                    .unwrap_or(0)
        );
        if let Some(t) = f["tasks"].get(c["id"].as_str().unwrap_or("")) {
            if t["submissions"].as_array().is_some_and(|v| !v.is_empty()) {
                submitted += 1;
            }
            match t["status"].as_str() {
                Some("approved") => approved += 1,
                Some("rejected") => rejected += 1,
                Some("supplement" | "supplement_required") => supplemented += 1,
                _ => {}
            }
            if let (Some(start), Some(end)) = (
                at(t, "submitted_at"),
                at(t, "reviewed_at").or_else(|| {
                    t["decisions"]
                        .as_array()
                        .and_then(|d| d.last())
                        .and_then(|d| at(d, "at"))
                }),
            ) && end >= start
            {
                durations.push(end - start);
            }
        }
    }
    let mut entries = vec![];
    for e in f["entries"].as_array().into_iter().flatten() {
        let src = e["source"].as_str().unwrap_or("");
        if !ids.contains(src) || !in_range(at(e, "at")) {
            continue;
        }
        entries.push(json!({"id":e["id"],"key":e["key"],"source":e["source"],"kind":e["kind"],"amount_cents":e["amount_cents"],"at":e["at"]}));
        if let Some(c) = claims.iter().find(|c| c["id"] == src) {
            let r = products
                .get_mut(c["product_id"].as_str().unwrap_or(""))
                .unwrap();
            let n = e["amount_cents"].as_i64().unwrap_or(0);
            let (key, delta) = match e["kind"].as_str() {
                Some("confirmed") => ("confirmed_cents", n),
                Some("increase") => ("adjusted_cents", n),
                Some("decrease") => ("adjusted_cents", -n),
                _ => ("adjusted_cents", 0),
            };
            r[key] = json!(r[key].as_i64().unwrap_or(0) + delta);
            r["entry_ids"].as_array_mut().unwrap().push(e["id"].clone());
        }
    }
    let mut applications = vec![];
    let mut progress: BTreeMap<String, Value> = BTreeMap::new();
    for app in f["applications"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.values())
    {
        if !owner_allowed(s, a, app["owner"].as_str().unwrap_or(""))
            || !in_range(at(app, "created_at"))
        {
            continue;
        }
        if !product.is_empty() || !source.is_empty() {
            let linked = f["entries"].as_array().into_iter().flatten().any(|e| {
                e["key"] == format!("withdrawal:{}:hold", app["id"].as_str().unwrap_or(""))
                    && e["detail"]["sources"].as_array().is_some_and(|v| {
                        v.iter()
                            .any(|r| ids.contains(r["source"].as_str().unwrap_or("")))
                    })
            });
            if !linked {
                continue;
            }
        }
        let status = app["status"].as_str().unwrap_or("unknown");
        let r = progress
            .entry(status.into())
            .or_insert(json!({"count":0,"amount_cents":0}));
        r["count"] = json!(r["count"].as_i64().unwrap_or(0) + 1);
        r["amount_cents"] = json!(
            r["amount_cents"].as_i64().unwrap_or(0) + app["amount_cents"].as_i64().unwrap_or(0)
        );
        applications.push(json!({"id":app["id"],"owner":app["owner"],"status":app["status"],"amount_cents":app["amount_cents"],"method":app["receipt_snapshot"]["method"],"created_at":app["created_at"],"paid_at":app["registration"]["paid_at"],"recorded_at":app["registration"]["recorded_at"]}));
    }
    // Settlement allocation follows the original hold sources, never a later receipt or rule.
    for app in &applications {
        if app["status"] != "settled" {
            continue;
        }
        for e in
            f["entries"].as_array().into_iter().flatten().filter(|e| {
                e["key"] == format!("withdrawal:{}:hold", app["id"].as_str().unwrap_or(""))
            })
        {
            for row in e["detail"]["sources"].as_array().into_iter().flatten() {
                if let Some(c) = claims.iter().find(|c| c["id"] == row["source"]) {
                    let r = products
                        .get_mut(c["product_id"].as_str().unwrap_or(""))
                        .unwrap();
                    r["settled_cents"] = json!(
                        r["settled_cents"].as_i64().unwrap_or(0)
                            + row["amount_cents"].as_i64().unwrap_or(0)
                    );
                }
            }
        }
    }
    let visits = f["touches"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.values())
        .filter(|t| {
            allowed(s, a, t["product"].as_str().unwrap_or(""))
                && (product.is_empty() || t["product"] == product)
                && (source.is_empty() || t["promoter"] == source)
                && in_range(at(t, "time").or_else(|| at(t, "at")))
        })
        .count();
    let registrations = f["users"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|u| {
            owner_allowed(s, a, u["id"].as_str().unwrap_or(""))
                && (source.is_empty() || u["source"] == source)
                && (product.is_empty() || u["source_product"] == product)
                && in_range(at(u, "registered_at"))
        })
        .count();
    let pending = f["tasks"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.values())
        .filter(|t| {
            ids.contains(t["id"].as_str().unwrap_or(""))
                && matches!(
                    t["status"].as_str(),
                    Some("submitted" | "reviewing" | "under_review" | "awaiting_verification")
                )
        })
        .count();
    let result = json!({"watermark":now,"timezone":"UTC","from":from,"until":until,"product_id":product,"source":source,"funnel":{"visits":visits,"registrations":registrations,"claims":claims.len(),"submitted_tasks":submitted,"approved_tasks":approved,"settled_applications":applications.iter().filter(|a|a["status"]=="settled").count()},"definitions":{"cohort":"按领取时间确定任务队列；提交与审核按此水位的当前状态；登记按申请提交时间筛选","dedup":"访问按服务端触点编号，注册按用户编号，领取与成果按任务编号，结算按申请编号","approval_denominator":approved+rejected,"approval_numerator":approved,"withdrawn":"撤回单不计入审核通过率分母","financial":"仅内部标记数字与人工登记，不是官方支付账单"},"review":{"pending":pending,"supplement_required":supplemented,"supplement_denominator":submitted,"processing_seconds":durations},"products":products.values().collect::<Vec<_>>(),"entries":entries,"settlement_progress":progress,"applications":applications});
    require_safe(&result)?;
    Ok(result)
}
fn require_safe(v: &Value) -> Result<()> {
    match v {
        Value::Number(n) => {
            if let Some(n) = n.as_i64() {
                require(
                    (-crate::shared::MAX..=crate::shared::MAX).contains(&n),
                    "报表汇总超出整数精度范围",
                )?
            }
        }
        Value::Array(a) => {
            for v in a {
                require_safe(v)?
            }
        }
        Value::Object(o) => {
            for v in o.values() {
                require_safe(v)?
            }
        }
        _ => {}
    }
    Ok(())
}
