use crate::campaigns::south::port::DomainPort;
use crate::shared::{Failure, MAX, Result, require};
use serde_json::{Value, json};
pub fn report<P: DomainPort + ?Sized>(s: &P, r: &Value) -> Result<Value> {
    let f = s.facts();
    let empty = serde_json::Map::new();
    let claims = f["claims"].as_object().unwrap_or(&empty);
    let rows: Vec<_> = claims
        .values()
        .filter(|t| {
            r["products"]
                .as_array()
                .is_some_and(|p| p.contains(&t["product_id"]))
        })
        .collect();
    let ids: std::collections::BTreeSet<_> = rows.iter().filter_map(|t| t["id"].as_str()).collect();
    let mut executing = 0_i128;
    let mut pending = 0_i128;
    let mut approved = 0;
    for c in &rows {
        let t = f["tasks"].get(c["id"].as_str().unwrap_or("")).unwrap_or(c);
        if t["status"] == "approved" || t["status"] == "disputed" {
            approved += 1;
        } else if t["reservation_released"] != true
            && !matches!(
                t["status"].as_str(),
                Some("abandoned" | "expired" | "rejected" | "invalidated")
            )
        {
            let n = t["reserved_cents"]
                .as_i64()
                .or_else(|| t["task"]["commission_cents"].as_i64())
                .unwrap_or(0);
            executing += i128::from(n);
            if matches!(
                t["status"].as_str(),
                Some("submitted" | "reviewing" | "awaiting_verification" | "supplement")
            ) {
                pending += i128::from(n);
            }
        }
    }
    let mut confirmed = 0_i128;
    let mut held = 0_i128;
    let mut settled = 0_i128;
    let mut disputed = 0_i128;
    let mut applications = std::collections::BTreeSet::new();
    for e in f["entries"].as_array().into_iter().flatten() {
        if ids.contains(e["source"].as_str().unwrap_or("")) {
            let n = i128::from(e["amount_cents"].as_i64().unwrap_or(0));
            match e["kind"].as_str() {
                Some("confirmed" | "increase") => confirmed += n,
                Some("decrease") => confirmed -= n,
                Some("dispute_hold") => disputed += n,
                Some("dispute_release") => disputed -= n,
                _ => (),
            }
        }
        if e["kind"] == "hold" {
            let parts: Vec<_> = e["detail"]["sources"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|p| ids.contains(p["source"].as_str().unwrap_or("")))
                .collect();
            let n: i128 = parts
                .iter()
                .map(|p| i128::from(p["amount_cents"].as_i64().unwrap_or(0)))
                .sum();
            if n > 0 {
                let id = e["source"].as_str().unwrap_or("");
                let app = &f["applications"][id];
                if app["status"] == "settled" {
                    settled += n;
                } else if !matches!(app["status"].as_str(), Some("cancelled" | "rejected")) {
                    held += n;
                    applications.insert(id);
                }
            }
        }
    }
    let used = executing + confirmed;
    let available = confirmed - held - settled - disputed;
    require(
        [
            executing, pending, confirmed, held, settled, used, disputed, available,
        ]
        .iter()
        .all(|n| n.abs() <= i128::from(MAX)),
        "活动汇总超出安全整数",
    )?;
    Ok(
        json!({"id":r["id"],"claimed":rows.len(),"approved":approved,"executing_cents":executing as i64,"pending_cents":pending as i64,"confirmed_cents":confirmed as i64,"available_cents":available as i64,"held_cents":held as i64,"settled_cents":settled as i64,"used_cents":used as i64,"applications":applications,"notice":"活动业务统计与计划数字，非银行余额或真实出资账户"}),
    )
}
pub fn is_read(section: &str) -> bool {
    matches!(section, "campaigns" | "campaign_report")
}
pub fn read<P: DomainPort + ?Sized>(
    s: &P,
    a: &str,
    section: &str,
    target: &str,
    _now: i64,
) -> Result<Value> {
    let permitted = |id: &str| {
        s.authorize(a, "operator", id)
            .or_else(|_| s.authorize(a, "auditor", id))
    };
    if section == "campaign_report" {
        permitted(target)?;
        let row = s
            .data()
            .records
            .get(target)
            .ok_or(Failure("NOT_FOUND", "活动不存在"))?;
        return report(s, row);
    }
    let mut rows = vec![];
    for r in s.data().records.values() {
        if permitted(r["id"].as_str().unwrap_or("")).is_ok() {
            let mut row = r.clone();
            row["statistics"] = report(s, r)?;
            rows.push(row);
        }
    }
    // Require a role even for an empty list.
    if rows.is_empty() {
        permitted(target)?;
    }
    Ok(json!(rows))
}
