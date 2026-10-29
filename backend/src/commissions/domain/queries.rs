use super::{MAX, require};
use crate::commissions::south::port::DomainPort;
use crate::shared::{Failure, Result};
use serde_json::{Value, json};
pub fn summary<P: DomainPort + ?Sized>(s: &P, owner: &str) -> Result<Value> {
    let mut confirmed = 0_i128;
    let mut gross_confirmed = 0_i128;
    let mut adjustment = 0_i128;
    let mut held = 0_i128;
    let mut settled = 0_i128;
    let mut disputed = 0_i128;
    let mut pending = 0_i128;
    let mut version = 0_u64;
    for e in s
        .commissions()
        .entries
        .iter()
        .filter(|e| e["owner"] == owner)
    {
        version += 1;
        let n = i128::from(
            e["amount_cents"]
                .as_i64()
                .ok_or(Failure("LEDGER_INCONSISTENT", "流水金额无效，请核查"))?,
        );
        match e["kind"].as_str().unwrap_or("") {
            "confirmed" => {
                confirmed += n;
                gross_confirmed += n;
            }
            "increase" => {
                confirmed += n;
                adjustment += n;
            }
            "decrease" => {
                confirmed -= n;
                adjustment -= n;
            }
            "hold" => held += n,
            "release" => held -= n,
            "settled" => {
                held -= n;
                settled += n;
            }
            "dispute_hold" => disputed += n,
            "dispute_release" => disputed -= n,
            _ => {}
        }
    }
    for t in s.tasks_instances().values().filter(|t| {
        t["owner"] == owner
            && matches!(
                t["status"].as_str(),
                Some("submitted" | "reviewing" | "awaiting_verification" | "supplement")
            )
    }) {
        pending += i128::from(t["reserved_cents"].as_i64().unwrap_or(0));
    }
    let available = confirmed - held - settled - disputed;
    require(
        [
            confirmed,
            gross_confirmed,
            adjustment,
            held,
            settled,
            disputed,
            pending,
            available,
        ]
        .iter()
        .all(|n| n.abs() <= i128::from(MAX)),
        "汇总超出安全整数范围",
    )?;
    let applications: i128 = s
        .withdrawals_applications()
        .values()
        .filter(|a| {
            a["owner"] == owner
                && !matches!(
                    a["status"].as_str(),
                    Some("rejected" | "cancelled" | "settled")
                )
        })
        .map(|a| i128::from(a["amount_cents"].as_i64().unwrap_or(0)))
        .sum();
    let source_totals = super::sources(s, owner)?;
    let valid_sources = source_totals.values().all(|n| *n >= 0)
        && source_totals.values().map(|n| i128::from(*n)).sum::<i128>() == available;
    let valid_confirmations = s
        .tasks_confirmations()
        .iter()
        .filter(|(_, c)| c["owner"] == owner)
        .all(|(id, c)| c["claim_id"] == id.as_str());
    let valid_holds = s
        .withdrawals_applications()
        .values()
        .filter(|a| a["owner"] == owner)
        .all(|a| {
            let id = a["id"].as_str().unwrap_or("");
            let mut original = 0_i128;
            let mut consumed = 0_i128;
            let mut hold_count = 0_u64;
            let mut valid_amounts = true;
            let held = s
                .commissions()
                .entries
                .iter()
                .filter(|e| e["owner"] == owner && e["source"] == id)
                .fold(0_i128, |n, e| {
                    let amount = i128::from(e["amount_cents"].as_i64().unwrap_or(-1));
                    valid_amounts &= amount >= 0;
                    match e["kind"].as_str() {
                        Some("hold") => {
                            original += amount;
                            hold_count += 1;
                        }
                        Some("release" | "settled") => consumed += amount,
                        _ => {}
                    }
                    n + match e["kind"].as_str() {
                        Some("hold") => i128::from(e["amount_cents"].as_i64().unwrap_or(0)),
                        Some("release" | "settled") => {
                            -i128::from(e["amount_cents"].as_i64().unwrap_or(0))
                        }
                        _ => 0,
                    }
                });
            valid_amounts
                && hold_count == 1
                && original == i128::from(a["amount_cents"].as_i64().unwrap_or(0))
                && consumed <= original
                && held
                    == if matches!(
                        a["status"].as_str(),
                        Some("rejected" | "cancelled" | "settled")
                    ) {
                        0
                    } else {
                        i128::from(a["amount_cents"].as_i64().unwrap_or(0))
                    }
        });
    let consistent = valid_holds
        && valid_sources
        && valid_confirmations
        && available >= 0
        && held >= 0
        && settled >= 0
        && disputed >= 0
        && applications == held;
    Ok(
        json!({"owner":owner,"currency":"CNY","version":version,"pending_cents":pending as i64,"confirmed_cents":confirmed as i64,"gross_confirmed_cents":gross_confirmed as i64,"adjustment_cents":adjustment as i64,"available_cents":available as i64,"held_cents":held as i64,"disputed_cents":disputed as i64,"settled_cents":settled as i64,"consistent":consistent,"notice":"仅为平台佣金标记数字，非存款余额；付款由人员线下处理"}),
    )
}
pub fn read<P: DomainPort + ?Sized>(
    s: &P,
    actor: &str,
    section: &str,
    target: &str,
) -> Result<Value> {
    s.user(actor)?;
    let (owner, cursor, kind) = if section == "commission_source" {
        let (owner, source) = target
            .split_once('|')
            .ok_or(Failure("INVALID_INPUT", "请指定用户及来源编号"))?;
        (owner, 0, source)
    } else if section == "commission_entries" {
        let mut parts = target.split('|');
        (
            parts.next().unwrap_or(actor),
            parts
                .next()
                .unwrap_or("0")
                .parse::<usize>()
                .map_err(|_| Failure("INVALID_INPUT", "分页游标无效"))?,
            parts.next().unwrap_or(""),
        )
    } else {
        (target, 0, "")
    };
    if owner != actor
        && s.authorize(actor, "settlement_reviewer", owner).is_err()
        && s.authorize(actor, "verifier", owner).is_err()
    {
        s.authorize(actor, "auditor", owner)?;
    }
    match section {
        "commission_summary" => summary(s, owner),
        "commission_source" => {
            let records: Vec<_> = s
                .commissions()
                .entries
                .iter()
                .filter(|e| e["owner"] == owner && (e["source"] == kind || e["id"] == kind))
                .collect();
            if records.is_empty() {
                return Err(Failure("NOT_FOUND", "该用户没有对应数字来源"));
            }
            let task=s.tasks_instances().get(kind).filter(|t|t["owner"]==owner).map(|t|json!({"id":kind,"product_id":t["product_id"],"status":t["status"],"price":t["price"],"calculation":s.tasks_confirmations().get(kind).map(|c|c["calculation"].clone())}));
            let applications:Vec<_>=s.withdrawals_applications().values().filter(|a|a["owner"]==owner&&(a["id"]==kind||super::hold_sources(s,a["id"].as_str().unwrap_or("")).as_array().is_some_and(|sources|sources.iter().any(|r|r["source"]==kind)))).map(|a|json!({"id":a["id"],"status":a["status"],"amount_cents":a["amount_cents"],"rule_version":a["rule"]["version"],"receipt_version":a["receipt_snapshot"]["version"]})).collect();
            Ok(
                json!({"owner":owner,"source":kind,"task":task,"applications":applications,"records":records}),
            )
        }
        "commission_entries" => {
            // Index is an append-only high-water cursor, unaffected by newer inserted events.
            let rows: Vec<_> = s
                .commissions()
                .entries
                .iter()
                .enumerate()
                .rev()
                .filter(|(index, e)| {
                    (cursor == 0 || *index < cursor)
                        && e["owner"] == owner
                        && (kind.is_empty() || e["kind"] == kind)
                })
                .collect();
            let items: Vec<_> = rows.iter().take(50).map(|(_, e)| *e).collect();
            let next = if rows.len() > 50 {
                rows.get(49).map(|(index, _)| *index)
            } else {
                None
            };
            Ok(
                json!({"items":items,"next_cursor":next,"total":s.commissions().entries.iter().filter(|e|e["owner"]==owner&&(kind.is_empty()||e["kind"]==kind)).count()}),
            )
        }
        "commission_proposals" => Ok(json!(
            s.commissions()
                .proposals
                .values()
                .filter(|p| p["owner"] == owner || p["new_owner"] == owner)
                .collect::<Vec<_>>()
        )),
        "commission_disputes" => Ok(json!(
            s.commissions()
                .disputes
                .values()
                .filter(|p| p["owner"] == owner)
                .collect::<Vec<_>>()
        )),
        _ => Err(Failure("NOT_FOUND", "佣金查询不存在")),
    }
}
