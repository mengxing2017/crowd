use crate::risk::south::port::DomainPort;
use crate::shared::{Failure, Result, number, require, text, version};
use serde_json::{Value, json};
pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "risk_rule_publish" | "risk_scan" | "risk_case_decide" | "risk_case_open"
    )
}
pub fn access<P: DomainPort + ?Sized>(s: &P, a: &str, action: &str, i: &Value) -> Result<()> {
    let target = if action == "risk_case_open" {
        text(i, "owner")?
    } else if action == "risk_case_decide" {
        s.data()
            .records
            .get(text(i, "id")?)
            .and_then(|r| r["owner"].as_str())
            .ok_or(Failure("NOT_FOUND", "风险处置单不存在"))?
    } else {
        text(i, "id")?
    };
    s.authorize(a, "risk_manager", target)
}
pub fn command<P: DomainPort + ?Sized>(
    s: &mut P,
    a: &str,
    action: &str,
    i: &Value,
    id: &str,
    now: i64,
) -> Result<Value> {
    access(s, a, action, i)?;
    text(i, "reason")?;
    if action == "risk_case_open" {
        let owner = text(i, "owner")?;
        let rule = s
            .data()
            .records
            .get(text(i, "rule_id")?)
            .cloned()
            .ok_or(Failure("NOT_FOUND", "风险规则不存在"))?;
        require(
            rule["kind"] == "rule"
                && rule["enabled"] == true
                && number(&rule, "starts_at")? <= now
                && number(&rule, "ends_at")? > now,
            "规则当前未生效",
        )?;
        require(s.facts()["receipts"][owner].is_array(), "用户不存在")?;
        text(i, "business_id")?;
        text(i, "evidence_summary")?;
        require(
            i["authorized_signals"] == true,
            "须确认信号来自已授权业务范围，不以单一相同网络认定违规",
        )?;
        let case = json!({"id":id,"kind":"case","version":1,"owner":owner,"business_id":i["business_id"],"signal":rule["signal"],"rule_id":rule["id"],"rule_version":rule["version"],"rule_snapshot":rule,"status":"pending_review","detected_by":a,"reason":i["reason"],"evidence_summary":i["evidence_summary"],"at":now,"notice":"人工核查信号，不是违规认定，可通过客服申诉"});
        s.data_mut().records.insert(id.into(), case.clone());
        return Ok(case);
    }
    if action == "risk_scan" {
        scan(s, now);
        return Ok(
            json!({"id":id,"status":"scanned","count":s.data().records.values().filter(|r|r["kind"]=="case").count()}),
        );
    }
    let key = text(i, "id")?;
    let old = s.data().records.get(key).cloned();
    let mut r = old.clone().unwrap_or(json!({"id":key,"version":0}));
    version(&r, i)?;
    match action {
        "risk_rule_publish" => {
            require(
                matches!(
                    i["signal"].as_str(),
                    Some(
                        "claim_frequency"
                            | "duplicate_evidence"
                            | "visit_frequency"
                            | "receipt_shared"
                            | "withdrawal_frequency"
                            | "account_change"
                            | "account_association"
                    )
                ),
                "未知风险信号",
            )?;
            text(i, "purpose")?;
            text(i, "evaluation")?;
            require(
                number(i, "starts_at")? < number(i, "ends_at")?,
                "生效期间无效",
            )?;
            require(i["enabled"].is_boolean(), "请选择是否启用")?;
            if matches!(
                i["signal"].as_str(),
                Some("claim_frequency" | "visit_frequency" | "withdrawal_frequency")
            ) {
                require(
                    number(i, "window_seconds")? > 0 && number(i, "threshold")? > 0,
                    "频率规则须配置正整数窗口和阈值",
                )?;
            }
            // A signal may create a review case, never an automatic financial penalty or mass ban.
            r = json!({"id":key,"version":r["version"],"kind":"rule","signal":i["signal"],"purpose":i["purpose"],"evaluation":i["evaluation"],"enabled":i["enabled"],"starts_at":i["starts_at"],"ends_at":i["ends_at"],"window_seconds":i["window_seconds"],"threshold":i["threshold"],"reason":i["reason"],"actor":a});
        }
        "risk_case_decide" => {
            require(r["kind"] == "case", "请选择风险处置单")?;
            let decision = text(i, "decision")?;
            require(
                matches!(decision, "observe" | "restrict" | "release" | "hold_source"),
                "处置结论无效",
            )?;
            if decision == "hold_source" {
                require(
                    r["signal"] != "visit_frequency" && r["signal"] != "account_association",
                    "低等级访问信号或单一关联信号不可直接暂缓数字",
                )?;
                let owner = text(&r, "owner")?.to_owned();
                let source = text(i, "source")?;
                let amount = number(i, "amount_cents")?;
                require(amount > 0, "请指定正整数分与真实数字来源")?;
                require(
                    r["dispute_id"].is_null(),
                    "本风险单已经存在数字暂缓，不可重复",
                )?;
                let dispute = format!("risk:{}", r["id"].as_str().unwrap_or(""));
                s.hold_source(a, &owner, source, amount, &dispute, text(i, "reason")?, now)?;
                r["dispute_id"] = json!(dispute);
                r["status"] = json!("digital_review");
            } else if decision == "restrict" {
                require(
                    matches!(i["scope"].as_str(), Some("task_claim" | "withdrawal_apply")),
                    "限制仅可作用于指定领取或新申请，不能删除佣金",
                )?;
                require(
                    r["signal"] != "visit_frequency" && r["signal"] != "account_association",
                    "访问或单一关联信号只允许人工观察，不能直接限制账号",
                )?;
                require(number(i, "expires_at")? > now, "限制必须指定有效期")?;
                require(r["detected_by"] != a, "限制须独立人员复核")?;
                r["status"] = json!("restricted");
                r["scope"] = i["scope"].clone();
                r["expires_at"] = i["expires_at"].clone();
            } else {
                if decision == "release"
                    && let Some(dispute) = r["dispute_id"].as_str()
                {
                    s.release_source(a, dispute, text(i, "reason")?, now)?;
                }
                r["status"] = json!(if decision == "release" {
                    "released"
                } else {
                    "observing"
                });
            }
            r["reviewer"] = json!(a);
            r["resolution"] = i["reason"].clone();
        }
        _ => return Err(Failure("NOT_FOUND", "风险操作不存在")),
    }
    r["version"] = json!(r["version"].as_u64().unwrap_or(0) + 1);
    r["updated_at"] = json!(now);
    s.data_mut()
        .history
        .push(json!({"id":id,"action":action,"actor":a,"at":now,"before":old,"after":r}));
    s.data_mut().records.insert(key.into(), r.clone());
    Ok(r)
}
fn hit<P: DomainPort + ?Sized>(s: &mut P, rule: &Value, owner: &str, business: &str, now: i64) {
    let id = format!(
        "risk:{}:{}:{owner}:{business}",
        rule["id"].as_str().unwrap_or(""),
        rule["version"]
    );
    s.data_mut().records.entry(id.clone()).or_insert(json!({"id":id,"kind":"case","owner":owner,"business_id":business,"signal":rule["signal"],"rule_id":rule["id"],"rule_version":rule["version"],"rule_snapshot":rule,"status":"pending_review","version":1,"at":now,"detected_by":"risk-detector","notice":"风险信号仅供人工核查，不代表违规认定；可通过客服工单申诉"}));
}
pub fn scan<P: DomainPort + ?Sized>(s: &mut P, now: i64) {
    let facts = s.facts();
    let rules: Vec<_> = s
        .data()
        .records
        .values()
        .filter(|r| {
            r["kind"] == "rule"
                && r["enabled"] == true
                && r["starts_at"].as_i64().unwrap_or(i64::MAX) <= now
                && r["ends_at"].as_i64().unwrap_or(0) > now
        })
        .cloned()
        .collect();
    for rule in rules {
        let signal = rule["signal"].as_str().unwrap_or("");
        let mut groups: std::collections::BTreeMap<String, Vec<Value>> =
            std::collections::BTreeMap::new();
        if signal == "receipt_shared" {
            for (owner, receipts) in facts["receipts"].as_object().into_iter().flatten() {
                let inactive: std::collections::BTreeSet<_> = receipts
                    .as_array()
                    .into_iter()
                    .flatten()
                    .flat_map(|r| [r["disables"].as_str(), r["cleans"].as_str()])
                    .flatten()
                    .collect();
                let mut latest = std::collections::BTreeMap::new();
                for r in receipts.as_array().into_iter().flatten() {
                    if let Some(method) = r["method"].as_str() {
                        latest.insert(method, r);
                    }
                }
                for r in latest.values().filter(|r| {
                    r["status"] == "confirmed" && !inactive.contains(r["id"].as_str().unwrap_or(""))
                }) {
                    // Fingerprints stay in memory only. Raw receipt data never enters a case or audit.
                    let fingerprint = if r["method"] == "alipay" {
                        r["account"].as_str().unwrap_or("").trim().to_lowercase()
                    } else {
                        r["sha256"]
                            .as_str()
                            .or_else(|| r["file_id"].as_str())
                            .unwrap_or("")
                            .into()
                    };
                    if !fingerprint.is_empty() {
                        groups
                            .entry(fingerprint)
                            .or_default()
                            .push(json!({"owner":owner,"id":r["id"]}));
                    }
                }
            }
        } else {
            let collection = match signal {
                "claim_frequency" => "claims",
                "visit_frequency" => "touches",
                "withdrawal_frequency" | "account_change" => "applications",
                _ => "tasks",
            };
            for r in facts[collection]
                .as_object()
                .into_iter()
                .flat_map(|m| m.values())
            {
                let owner = r["owner"]
                    .as_str()
                    .or_else(|| r["promoter"].as_str())
                    .unwrap_or("");
                if owner.is_empty() {
                    continue;
                }
                let k = if signal == "duplicate_evidence" {
                    r["receipt"]["evidence"]["fields"]["description"]
                        .as_str()
                        .or_else(|| r["receipt"]["evidence"]["description"].as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string()
                } else {
                    owner.to_owned()
                };
                if k.is_empty() {
                    continue;
                }
                let at = r["locked_at"]
                    .as_i64()
                    .or_else(|| r["created_at"].as_i64())
                    .or_else(|| r["at"].as_i64())
                    .or_else(|| r["time"].as_i64())
                    .unwrap_or(0);
                if !matches!(
                    signal,
                    "claim_frequency" | "visit_frequency" | "withdrawal_frequency"
                ) || at >= now - rule["window_seconds"].as_i64().unwrap_or(0)
                {
                    groups.entry(k).or_default().push(
                        json!({"owner":owner,"id":r["id"],"receipt":r["receipt_snapshot"]["id"]}),
                    );
                }
            }
        }
        for rows in groups.values() {
            let owners: std::collections::BTreeSet<_> =
                rows.iter().filter_map(|r| r["owner"].as_str()).collect();
            let hit_rule = match signal {
                "receipt_shared" => owners.len() > 1,
                "duplicate_evidence" => rows.len() > 1,
                "account_change" => {
                    rows.iter()
                        .filter_map(|r| r["receipt"].as_str())
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        > 1
                }
                "account_association" => false,
                _ => rows.len() as i64 > rule["threshold"].as_i64().unwrap_or(i64::MAX),
            };
            if hit_rule {
                for r in rows {
                    hit(
                        s,
                        &rule,
                        r["owner"].as_str().unwrap_or(""),
                        r["id"].as_str().unwrap_or(""),
                        now,
                    );
                }
            }
        }
    }
}
pub fn check<P: DomainPort + ?Sized>(s: &P, owner: &str, action: &str, now: i64) -> Result<()> {
    require(
        !s.data().records.values().any(|r| {
            r["kind"] == "case"
                && r["owner"] == owner
                && r["status"] == "restricted"
                && r["scope"] == action
                && r["expires_at"].as_i64().unwrap_or(0) > now
        }),
        "该操作因风险核查暂受限，可在客服工单中申诉",
    )?;
    // Configured frequency gates use existing successful actions only and never alter them.
    if matches!(action, "task_claim" | "withdrawal_apply") {
        let f = s.facts();
        let (collection, signal) = if action == "task_claim" {
            ("claims", "claim_frequency")
        } else {
            ("applications", "withdrawal_frequency")
        };
        for rule in s.data().records.values().filter(|r| {
            r["kind"] == "rule"
                && r["signal"] == signal
                && r["enabled"] == true
                && r["starts_at"].as_i64().unwrap_or(i64::MAX) <= now
                && r["ends_at"].as_i64().unwrap_or(0) > now
        }) {
            let start = now - rule["window_seconds"].as_i64().unwrap_or(0);
            let count = f[collection]
                .as_object()
                .into_iter()
                .flat_map(|m| m.values())
                .filter(|v| {
                    v["owner"] == owner
                        && v["locked_at"]
                            .as_i64()
                            .or_else(|| v["created_at"].as_i64())
                            .or_else(|| v["at"].as_i64())
                            .unwrap_or(0)
                            >= start
                })
                .count();
            require(
                (count as i64) < rule["threshold"].as_i64().unwrap_or(i64::MAX),
                "超过已发布规则的操作频率，请稍后重试或提交申诉",
            )?;
        }
    }
    Ok(())
}
