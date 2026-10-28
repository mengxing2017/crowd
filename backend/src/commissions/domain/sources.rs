use super::{MAX, require};
use crate::commissions::south::port::DomainPort;
use crate::shared::Result;
use serde_json::{Value, json};
use std::collections::BTreeMap;
// Attribute holds to their exact confirmation/adjustment sources; unrelated numbers remain usable.
pub fn sources<P: DomainPort + ?Sized>(s: &P, owner: &str) -> Result<BTreeMap<String, i64>> {
    let mut totals: BTreeMap<String, i128> = BTreeMap::new();
    for e in s
        .commissions()
        .entries
        .iter()
        .filter(|e| e["owner"] == owner)
    {
        let n = i128::from(e["amount_cents"].as_i64().unwrap_or(0));
        let kind = e["kind"].as_str().unwrap_or("");
        if matches!(
            kind,
            "confirmed" | "increase" | "decrease" | "dispute_hold" | "dispute_release"
        ) {
            *totals
                .entry(e["source"].as_str().unwrap_or("").into())
                .or_default() += if matches!(kind, "decrease" | "dispute_hold") {
                -n
            } else {
                n
            };
        } else if matches!(kind, "hold" | "release")
            && let Some(items) = e["detail"]["sources"].as_array()
        {
            for item in items {
                *totals
                    .entry(item["source"].as_str().unwrap_or("").into())
                    .or_default() += i128::from(item["amount_cents"].as_i64().unwrap_or(0))
                    * if kind == "hold" { -1 } else { 1 };
            }
        }
        // Settled keeps the original hold deducted from this source: it must never become reusable.
    }
    totals
        .into_iter()
        .map(|(key, n)| {
            require(n.abs() <= i128::from(MAX), "来源数字超出安全范围")?;
            Ok((key, n as i64))
        })
        .collect()
}
pub fn allocate<P: DomainPort + ?Sized>(s: &P, owner: &str, n: i64) -> Result<Value> {
    let mut left = n;
    let mut rows = vec![];
    for (source, available) in sources(s, owner)? {
        let take = available.max(0).min(left);
        if take > 0 {
            rows.push(json!({"source":source,"amount_cents":take}));
            left -= take;
        }
        if left == 0 {
            break;
        }
    }
    require(left == 0, "缺少可核对的有效佣金来源")?;
    Ok(json!(rows))
}
pub fn hold_sources<P: DomainPort + ?Sized>(s: &P, id: &str) -> Value {
    s.commissions()
        .entries
        .iter()
        .find(|e| e["key"] == format!("withdrawal:{id}:hold"))
        .map(|e| e["detail"]["sources"].clone())
        .unwrap_or(json!([]))
}
pub fn source_disputed<P: DomainPort + ?Sized>(s: &P, app: &Value) -> bool {
    hold_sources(s, app["id"].as_str().unwrap_or(""))
        .as_array()
        .is_some_and(|rows| {
            rows.iter().any(|row| {
                s.commissions().disputes.values().any(|d| {
                    d["owner"] == app["owner"]
                        && d["source"] == row["source"]
                        && d["status"] == "open"
                        && d["blocks_applications"] == true
                })
            })
        })
}
