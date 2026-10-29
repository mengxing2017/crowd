use crate::risk::south::port::DomainPort;
use crate::shared::Result;
use serde_json::{Value, json};
pub fn is_read(a: &str) -> bool {
    matches!(a, "risk_rules" | "risk_cases" | "my_risk")
}
pub fn read<P: DomainPort + ?Sized>(
    s: &P,
    a: &str,
    section: &str,
    target: &str,
    now: i64,
) -> Result<Value> {
    if section == "risk_rules" {
        s.authorize(a, "risk_manager", target)?;
        return Ok(json!(
            s.data()
                .records
                .values()
                .filter(|r| r["kind"] == "rule")
                .collect::<Vec<_>>()
        ));
    }
    let rows: Vec<_> = s
        .data()
        .records
        .values()
        .filter(|r| {
            r["kind"] == "case"
                && if section == "my_risk" {
                    r["owner"] == a
                } else {
                    s.authorize(a, "risk_manager", r["owner"].as_str().unwrap_or(""))
                        .is_ok()
                }
        })
        .map(|r| {
            let mut r = r.clone();
            r["effective"] =
                json!(r["status"] == "restricted" && r["expires_at"].as_i64().unwrap_or(0) > now);
            if section == "my_risk" {
                r.as_object_mut().unwrap().remove("rule_snapshot");
                r.as_object_mut().unwrap().remove("evidence_summary");
            }
            r
        })
        .collect();
    if section != "my_risk" && rows.is_empty() {
        s.authorize(a, "risk_manager", target)?;
    }
    Ok(json!(rows))
}
