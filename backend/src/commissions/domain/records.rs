use super::{MAX, require, summary};
use crate::commissions::south::port::DomainPort;
use crate::shared::{Result, text};
use serde_json::{Value, json};
// Every change has a stable business key; callers share the outer locked transaction.
// Immutable event metadata is explicit at each transactional call site.
#[allow(clippy::too_many_arguments)]
pub fn append<P: DomainPort + ?Sized>(
    s: &mut P,
    key: &str,
    owner: &str,
    kind: &str,
    amount: i64,
    source: &str,
    actor: &str,
    now: i64,
    detail: Value,
) -> Result<Value> {
    if let Some(old) = s.commissions().entries.iter().find(|e| e["key"] == key) {
        require(
            old["owner"] == owner
                && old["kind"] == kind
                && old["amount_cents"] == amount
                && old["source"] == source,
            "业务流水键与原记录不一致",
        )?;
        return Ok(old.clone());
    }
    require((0..=MAX).contains(&amount), "流水金额超出安全整数范围")?;
    s.user(owner)?;
    let e = json!({"id":key,"key":key,"owner":owner,"kind":kind,"amount_cents":amount,"source":source,"actor":actor,"at":now,"currency":"CNY","detail":detail,"external_payment":false});
    s.commissions_mut().entries.push(e.clone());
    // Reject arithmetic overflow immediately; the application restores all mutations on error.
    summary(s, owner)?;
    Ok(e)
}
pub fn sync_tasks<P: DomainPort + ?Sized>(s: &mut P, actor: &str, now: i64) -> Result<()> {
    let tasks: Vec<Value> = s.tasks_instances().values().cloned().collect();
    for t in tasks {
        let id = text(&t, "id")?;
        let owner = text(&t, "owner")?;
        if !s
            .commissions()
            .entries
            .iter()
            .any(|e| e["key"] == format!("task:{id}:{owner}:pending"))
            && t["submissions"].as_array().is_some_and(|a| !a.is_empty())
        {
            let amount = t["reserved_cents"].as_i64().unwrap_or(0);
            append(
                s,
                &format!("task:{id}:{owner}:pending"),
                owner,
                "pending",
                amount,
                id,
                actor,
                now,
                json!({"price":t["price"],"estimate":true}),
            )?;
        }
        if let Some(c) = s
            .tasks_confirmations()
            .get(id)
            .filter(|c| c["claim_id"] == id && c["owner"] == owner)
            .cloned()
        {
            let amount = c["calculation"]["amount_cents"].as_i64().unwrap_or(0);
            append(
                s,
                &format!("task:{id}:{owner}:confirmed"),
                owner,
                "confirmed",
                amount,
                id,
                actor,
                now,
                json!({"price":c["price"],"calculation":c["calculation"],"confirmation":id}),
            )?;
        }
        if t["status"] == "disputed" {
            let key = format!("task:{id}:dispute");
            let existing = s.commissions().disputes.get(&key).cloned();
            if existing.as_ref().is_some_and(|d| d["status"] == "resolved") {
                continue;
            }
            let free = super::sources(s, owner)?
                .get(id)
                .copied()
                .unwrap_or(0)
                .max(0);
            let mut d=existing.unwrap_or(json!({"id":key,"owner":owner,"source":id,"amount_cents":0,"status":"open","reason":"任务结论争议；已占用或已结算部分进入核查，不自动追偿","responsible":actor,"at":now,"version":1,"blocks_applications":true}));
            let previous = d["amount_cents"].as_i64().unwrap_or(0);
            if free > 0 {
                append(
                    s,
                    &format!("{key}:{}", previous + free),
                    owner,
                    "dispute_hold",
                    free,
                    id,
                    actor,
                    now,
                    json!({"reason":d["reason"]}),
                )?;
                d["amount_cents"] = json!(previous + free);
            }
            s.commissions_mut().disputes.insert(key, d);
        }
    }
    reconcile_disputes(s, actor, now)?;
    Ok(())
}

fn reconcile_disputes<P: DomainPort + ?Sized>(s: &mut P, actor: &str, now: i64) -> Result<()> {
    let disputes: Vec<Value> = s
        .commissions()
        .disputes
        .values()
        .filter(|d| d["status"] == "open")
        .cloned()
        .collect();
    for mut d in disputes {
        let source = text(&d, "source")?;
        let owner = text(&d, "owner")?;
        let id = text(&d, "id")?.to_owned();
        let previous = d["amount_cents"].as_i64().unwrap_or(0);
        let requested = d["claimed_cents"].as_i64().unwrap_or(previous);
        let free = super::sources(s, owner)?
            .get(source)
            .copied()
            .unwrap_or(0)
            .max(0);
        let extra = (requested - previous).max(0).min(free);
        if extra > 0 {
            append(
                s,
                &format!("dispute:{id}:additional:{}", previous + extra),
                owner,
                "dispute_hold",
                extra,
                source,
                actor,
                now,
                json!({"reason":d["reason"],"released_application_still_disputed":true}),
            )?;
            d["amount_cents"] = json!(previous + extra);
            s.commissions_mut().disputes.insert(id, d);
        }
    }
    Ok(())
}
