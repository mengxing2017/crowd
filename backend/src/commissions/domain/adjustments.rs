use crate::commissions::south::port::DomainPort;
use crate::shared::{Failure, MAX, Result, number, require, text, version};
use serde_json::{Value, json};
pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "adjustment_propose"
            | "adjustment_locate"
            | "adjustment_review"
            | "adjustment_batch_preview"
            | "adjustment_batch_review"
    )
}
pub fn access<P: DomainPort + ?Sized>(s: &P, a: &str, action: &str, i: &Value) -> Result<()> {
    if action == "adjustment_batch_preview" {
        let items = i["items"]
            .as_array()
            .ok_or(Failure("INVALID_INPUT", "请选择调整单"))?;
        for item in items {
            let p = s
                .commissions()
                .adjustments
                .get(text(item, "id")?)
                .ok_or(Failure("NOT_FOUND", "调整单不存在"))?;
            s.authorize(a, "settlement_reviewer", text(p, "owner")?)?;
        }
        return Ok(());
    }
    if action == "adjustment_batch_review" {
        let batch = s
            .commissions()
            .adjustments
            .get(text(i, "id")?)
            .ok_or(Failure("NOT_FOUND", "批次不存在"))?;
        for item in batch["items"]
            .as_array()
            .ok_or(Failure("INVALID_INPUT", "批次无效"))?
        {
            let p = s
                .commissions()
                .adjustments
                .get(text(item, "id")?)
                .ok_or(Failure("NOT_FOUND", "调整单不存在"))?;
            s.authorize(a, "verifier", text(p, "owner")?)?;
        }
        return Ok(());
    }
    let owner = if matches!(action, "adjustment_review" | "adjustment_locate") {
        s.commissions()
            .adjustments
            .get(text(i, "id")?)
            .and_then(|r| r["owner"].as_str())
            .ok_or(Failure("NOT_FOUND", "调整单不存在"))?
    } else {
        text(i, "owner")?
    };
    s.authorize(
        a,
        if action == "adjustment_review" {
            "verifier"
        } else {
            "settlement_reviewer"
        },
        owner,
    )
}
fn impacts<P: DomainPort + ?Sized>(s: &P, p: &Value) -> Vec<Value> {
    s.withdrawals_applications().values().filter(|app|app["owner"]==p["owner"]&&super::hold_sources(s,app["id"].as_str().unwrap_or("")).as_array().is_some_and(|rows|rows.iter().any(|r|r["source"]==p["task_id"]))&& !matches!(app["status"].as_str(),Some("cancelled"|"rejected"))).map(|app|json!({"id":app["id"],"status":app["status"],"handler":app["handler"],"amount_cents":app["amount_cents"]})).collect()
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
    if action == "adjustment_batch_preview" {
        let items = i["items"]
            .as_array()
            .ok_or(Failure("INVALID_INPUT", "请选择调整单"))?;
        require(
            !items.is_empty() && items.len() <= 50,
            "批次须包含1至50个调整单",
        )?;
        let mut seen = std::collections::BTreeSet::new();
        let mut people = std::collections::BTreeSet::new();
        let mut sum = 0_i128;
        let mut apps = vec![];
        for item in items {
            let key = text(item, "id")?;
            require(seen.insert(key), "调整单重复")?;
            let p = s
                .commissions()
                .adjustments
                .get(key)
                .ok_or(Failure("NOT_FOUND", "调整单不存在"))?;
            version(p, item)?;
            require(
                p["kind"] == "adjustment"
                    && matches!(
                        p["status"].as_str(),
                        Some("pending" | "awaiting_resolution")
                    ),
                "调整单不在待复核状态",
            )?;
            people.insert(text(p, "owner")?.to_string());
            sum += i128::from(number(p, "amount_cents")?);
            apps.extend(impacts(s, p));
        }
        require(sum <= i128::from(MAX), "批次总数超出安全整数")?;
        let b = json!({"id":id,"kind":"batch","actor":a,"status":"previewed","version":1,"items":items,"affected_users":people.len(),"amount_cents":sum as i64,"applications":apps,"reason":i["reason"],"at":now});
        s.commissions_mut().adjustments.insert(id.into(), b.clone());
        return Ok(b);
    }
    if action == "adjustment_batch_review" {
        let key = text(i, "id")?;
        let mut b = s
            .commissions()
            .adjustments
            .get(key)
            .cloned()
            .ok_or(Failure("NOT_FOUND", "批次不存在"))?;
        version(&b, i)?;
        require(
            b["kind"] == "batch"
                && b["status"] == "previewed"
                && b["actor"] != a
                && i["confirmed"] == true,
            "批次须独立确认原预览",
        )?;
        for item in b["items"].as_array().unwrap() {
            let p = s
                .commissions()
                .adjustments
                .get(text(item, "id")?)
                .ok_or(Failure("NOT_FOUND", "调整单不存在"))?;
            version(p, item)?;
        }
        let mut results = vec![];
        for item in b["items"].as_array().unwrap() {
            results.push(command(s,a,"adjustment_review",&json!({"id":item["id"],"version":item["version"],"approve":true,"reason":i["reason"]}),id,now)?);
        }
        b["status"] = json!("reviewed");
        b["version"] = json!(2);
        b["reviewer"] = json!(a);
        b["results"] = json!(results);
        s.commissions_mut()
            .adjustments
            .insert(key.into(), b.clone());
        return Ok(b);
    }
    if action == "adjustment_locate" {
        let key = text(i, "id")?;
        let mut p = s
            .commissions()
            .adjustments
            .get(key)
            .cloned()
            .ok_or(Failure("NOT_FOUND", "依据单不存在"))?;
        version(&p, i)?;
        require(
            p["status"] == "pending_location",
            "仅未定位依据可以补充可信任务定位",
        )?;
        let taskid = text(i, "task_id")?;
        let task = s
            .tasks_instances()
            .get(taskid)
            .cloned()
            .ok_or(Failure("NOT_FOUND", "任务不存在"))?;
        require(task["owner"] == p["owner"], "定位任务归属不符")?;
        let confirmed = s.tasks_confirmations().get(taskid).cloned();
        let amount = if let Some(c) = &confirmed {
            let n = number(&p, "requested_cents")?;
            require(
                n > 0 && n <= number(&c["calculation"], "amount_cents")?,
                "调整数超出原确认范围",
            )?;
            n
        } else {
            0
        };
        p["task_id"] = json!(taskid);
        p["task_version"] = task["version"].clone();
        p["original"] = json!(confirmed);
        p["amount_cents"] = json!(amount);
        p["status"] = json!("pending");
        p["version"] = json!(p["version"].as_u64().unwrap_or(0) + 1);
        p["located_by"] = json!(a);
        p["location_reason"] = i["reason"].clone();
        s.commissions_mut()
            .adjustments
            .insert(key.into(), p.clone());
        return Ok(p);
    }
    if action == "adjustment_propose" {
        let owner = text(i, "owner")?;
        s.user(owner)?;
        let taskid = text(i, "task_id")?;
        let basis = text(i, "basis_key")?;
        text(i, "source")?;
        text(i, "evidence")?;
        require(
            !s.commissions()
                .adjustments
                .values()
                .any(|p| p["basis_key"] == basis),
            "同一失效或恢复依据已经登记",
        )?;
        let task = s.tasks_instances().get(taskid).cloned();
        if let Some(t) = &task {
            require(t["owner"] == owner, "任务不属于指定用户")?;
        }
        let confirmed = s.tasks_confirmations().get(taskid).cloned();
        let mut amount = if confirmed.is_some() {
            number(i, "amount_cents")?
        } else {
            0
        };
        require(amount >= 0, "调整数字不能为负")?;
        let mut kind = "decrease";
        if let Some(original) = i["restores"].as_str().filter(|v| !v.is_empty()) {
            let p = s
                .commissions()
                .adjustments
                .get(original)
                .ok_or(Failure("NOT_FOUND", "原调减单不存在"))?;
            require(
                p["owner"] == owner
                    && p["task_id"] == taskid
                    && p["status"] == "applied"
                    && p["direction"] == "decrease",
                "恢复必须关联已生效的原调减",
            )?;
            let restored: i128 = s
                .commissions()
                .adjustments
                .values()
                .filter(|r| r["restores"] == original && r["status"] != "rejected")
                .map(|r| i128::from(r["amount_cents"].as_i64().unwrap_or(0)))
                .sum();
            require(
                restored + i128::from(amount) <= i128::from(number(p, "amount_cents")?),
                "恢复数超过原调整或已有恢复申请",
            )?;
            kind = "restore";
        } else if let Some(c) = &confirmed {
            let total = number(&c["calculation"], "amount_cents")?;
            if !i["invalid_base_cents"].is_null() {
                let base = number(&c["calculation"], "base_cents")?;
                let invalid = number(i, "invalid_base_cents")?;
                require(
                    base > 0 && invalid > 0 && invalid <= base,
                    "部分失效基数不合法或缺少可信原基数",
                )?;
                let n = i128::from(total) * i128::from(invalid);
                let d = i128::from(base);
                let calc = match c["price"]["rounding"].as_str() {
                    Some("ceil") => (n + d - 1) / d,
                    Some("half_up") => (n + d / 2) / d,
                    _ => n / d,
                };
                amount = calc as i64;
            }
            require(amount > 0 && amount <= total, "调整数字超出原确认范围")?;
        }
        let p = json!({"id":id,"kind":"adjustment","direction":kind,"owner":owner,"task_id":taskid,"basis_key":basis,"source":i["source"],"evidence":i["evidence"],"amount_cents":amount,"requested_cents":i["amount_cents"],"invalid_base_cents":i["invalid_base_cents"],"restores":i["restores"],"ticket_id":i["ticket_id"],"task_version":task.as_ref().map(|t|t["version"].clone()),"original":confirmed,"reason":i["reason"],"actor":a,"at":now,"status":if task.is_some(){"pending"}else{"pending_location"},"version":1});
        s.commissions_mut().adjustments.insert(id.into(), p.clone());
        return Ok(p);
    }
    let key = text(i, "id")?;
    let mut p = s
        .commissions()
        .adjustments
        .get(key)
        .cloned()
        .ok_or(Failure("NOT_FOUND", "调整单不存在"))?;
    version(&p, i)?;
    require(
        p["kind"] == "adjustment"
            && matches!(
                p["status"].as_str(),
                Some("pending" | "awaiting_resolution")
            )
            && p["actor"] != a,
        "调整须独立复核且不可重复处理",
    )?;
    require(i["approve"].is_boolean(), "请选择复核结论")?;
    p["before_summary"] = super::summary(s, text(&p, "owner")?)?;
    if i["approve"] == true {
        let taskid_owned = text(&p, "task_id")?.to_owned();
        let owner_owned = text(&p, "owner")?.to_owned();
        let taskid = taskid_owned.as_str();
        let owner = owner_owned.as_str();
        let n = number(&p, "amount_cents")?;
        let task = s
            .tasks_instances()
            .get(taskid)
            .cloned()
            .ok_or(Failure("NOT_FOUND", "原任务不存在"))?;
        require(
            task["version"] == p["task_version"],
            "原任务版本变化，请重新登记依据",
        )?;
        if p["original"].is_null() {
            s.cancel_task(taskid, a, now)?;
            p["status"] = json!("applied");
            p["outcome"] = json!("pending_cancelled");
        } else if p["direction"] == "restore" {
            let original = text(&p, "restores")?;
            let restore_key = format!("restore:{original}:{key}");
            super::append(
                s,
                &restore_key,
                owner,
                "increase",
                n,
                taskid,
                a,
                now,
                p.clone(),
            )?;
            p["status"] = json!("applied");
        } else {
            let affected = impacts(s, &p);
            p["applications"] = json!(affected);
            if !affected.is_empty() {
                let dispute_id = format!("adjustment:{key}");
                if affected.iter().all(|app| app["status"] == "settled")
                    && s.commissions()
                        .disputes
                        .get(&dispute_id)
                        .is_some_and(|d| d["status"] == "resolved")
                {
                    p["status"] = json!("dispute_recorded");
                    p["outcome"] = json!(
                        "已登记结算争议已有明确人工处理结论，记录结案；不自动追偿或从未来数字抵扣"
                    );
                } else {
                    if !s.commissions().disputes.contains_key(&dispute_id) {
                        let free = super::sources(s, owner)?
                            .get(taskid)
                            .copied()
                            .unwrap_or(0)
                            .max(0);
                        let hold = n.min(free);
                        let d = json!({"id":dispute_id,"owner":owner,"source":taskid,"amount_cents":hold,"claimed_cents":n,"blocks_applications":true,"status":"open","reason":p["reason"],"responsible":a,"at":now,"version":1});
                        super::append(
                            s,
                            &format!("{dispute_id}:hold"),
                            owner,
                            "dispute_hold",
                            hold,
                            taskid,
                            a,
                            now,
                            d.clone(),
                        )?;
                        s.commissions_mut().disputes.insert(dispute_id, d);
                    }
                    for app in &affected {
                        if app["status"] != "settled" {
                            s.pause_application(text(app, "id")?, a, now)?;
                        }
                    }
                    p["status"] = json!("awaiting_resolution");
                    p["outcome"] = json!("人工核查在途或已结算记录；不自动追回，也不抵扣未来佣金");
                }
            } else {
                let dispute_id = format!("adjustment:{key}");
                if let Some(mut d) = s
                    .commissions()
                    .disputes
                    .get(&dispute_id)
                    .cloned()
                    .filter(|d| d["status"] == "open")
                {
                    let hold = d["amount_cents"].as_i64().unwrap_or(0);
                    super::append(
                        s,
                        &format!("{dispute_id}:release"),
                        owner,
                        "dispute_release",
                        hold,
                        taskid,
                        a,
                        now,
                        json!({"adjustment":key}),
                    )?;
                    d["status"] = json!("resolved");
                    s.commissions_mut().disputes.insert(dispute_id, d);
                }
                require(
                    super::sources(s, owner)?.get(taskid).copied().unwrap_or(0) >= n,
                    "原任务剩余可调整数字不足",
                )?;
                super::append(
                    s,
                    &format!("adjustment:{key}:decrease"),
                    owner,
                    "decrease",
                    n,
                    taskid,
                    a,
                    now,
                    p.clone(),
                )?;
                p["status"] = json!("applied");
            }
        }
    } else {
        require(
            p["status"] == "pending",
            "核查中调整须先处理关联争议，不可直接驳回释放",
        )?;
        p["status"] = json!("rejected");
    }
    p["version"] = json!(p["version"].as_u64().unwrap_or(0) + 1);
    p["reviewer"] = json!(a);
    p["review_reason"] = i["reason"].clone();
    p["reviewed_at"] = json!(now);
    p["after_summary"] = super::summary(s, text(&p, "owner")?)?;
    s.commissions_mut()
        .adjustments
        .insert(key.into(), p.clone());
    Ok(p)
}
pub fn read<P: DomainPort + ?Sized>(s: &P, a: &str, target: &str) -> Result<Value> {
    let owner = if target.is_empty() { a } else { target };
    if owner != a {
        s.authorize(a, "settlement_reviewer", owner)
            .or_else(|_| s.authorize(a, "verifier", owner))
            .or_else(|_| s.authorize(a, "auditor", owner))?;
    }
    Ok(json!(
        s.commissions()
            .adjustments
            .values()
            .filter(|p| p["owner"] == owner
                || p["kind"] == "batch"
                    && p["items"]
                        .as_array()
                        .is_some_and(|items| items.iter().any(|i| s
                            .commissions()
                            .adjustments
                            .get(i["id"].as_str().unwrap_or(""))
                            .is_some_and(|r| r["owner"] == owner))
                            && items.iter().all(|i| s
                                .commissions()
                                .adjustments
                                .get(i["id"].as_str().unwrap_or(""))
                                .is_some_and(|r| {
                                    let o = r["owner"].as_str().unwrap_or("");
                                    s.authorize(a, "settlement_reviewer", o)
                                        .or_else(|_| s.authorize(a, "verifier", o))
                                        .or_else(|_| s.authorize(a, "auditor", o))
                                        .is_ok()
                                }))))
            .collect::<Vec<_>>()
    ))
}
