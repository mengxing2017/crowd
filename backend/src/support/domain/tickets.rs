use crate::shared::{Failure, Result, number, require, text, version};
use crate::support::south::port::DomainPort;
use serde_json::{Value, json};
pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "ticket_create"
            | "ticket_assign"
            | "ticket_reply"
            | "ticket_close"
            | "ticket_reopen"
            | "support_policy"
    )
}
pub fn is_read(a: &str) -> bool {
    matches!(a, "tickets" | "support_statistics" | "support_policy")
}
pub fn access<P: DomainPort + ?Sized>(s: &P, a: &str, action: &str, i: &Value) -> Result<()> {
    if action == "support_policy" {
        return s.authorize(a, "rule_manager", "support");
    }
    if action == "ticket_create" {
        return Ok(());
    }
    let t = s
        .cases()
        .iter()
        .find(|t| t["id"] == i["id"] && t["kind"] == "ticket")
        .ok_or(Failure("NOT_FOUND", "工单不存在"))?;
    if t["owner"] == a && matches!(action, "ticket_reply" | "ticket_reopen") {
        return Ok(());
    }
    s.authorize(a, "support", text(t, "owner")?)
}
fn policy<P: DomainPort + ?Sized>(s: &P) -> Value {
    s.cases()
        .iter()
        .rev()
        .find(|t| t["kind"] == "support_policy")
        .cloned()
        .unwrap_or(json!({"version":0,"configured":false}))
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
    if action == "support_policy" {
        text(i, "reason")?;
        version(&policy(s), i)?;
        for field in ["appeal_seconds", "max_appeals", "reopen_seconds"] {
            require(number(i, field)? > 0, "申诉次数与期限必须显式配置为正整数")?;
        }
        require(i["independent"].is_boolean(), "请选择是否强制独立复核")?;
        let p = json!({"id":id,"kind":"support_policy","version":policy(s)["version"].as_u64().unwrap_or(0)+1,"configured":true,"appeal_seconds":i["appeal_seconds"],"max_appeals":i["max_appeals"],"reopen_seconds":i["reopen_seconds"],"independent":i["independent"],"reason":i["reason"],"actor":a,"at":now});
        s.cases_mut().push(p.clone());
        return Ok(p);
    }
    let f = s.facts();
    let p = policy(s);
    if action == "ticket_create" {
        let category = text(i, "category")?;
        text(i, "description")?;
        require(
            matches!(
                category,
                "registration"
                    | "promotion"
                    | "task"
                    | "commission"
                    | "withdrawal"
                    | "appeal"
                    | "risk"
            ),
            "工单分类无效",
        )?;
        let relation = i["related_id"].as_str().unwrap_or("");
        let mut original = Value::Null;
        if !relation.is_empty() {
            let found = match category {
                "task" | "appeal" => f["tasks"][relation].clone(),
                "withdrawal" => s
                    .withdrawals_applications()
                    .get(relation)
                    .cloned()
                    .unwrap_or(Value::Null),
                "commission" => s
                    .commissions_entries()
                    .iter()
                    .find(|e| e["id"] == relation || e["source"] == relation && e["owner"] == a)
                    .cloned()
                    .unwrap_or(Value::Null),
                "risk" => f["risk"][relation].clone(),
                "promotion" => f["codes"][relation].clone(),
                _ => Value::Null,
            };
            require(
                found.is_object() && found["owner"] == a,
                "关联业务不存在或不属于本人",
            )?;
            original = found;
        }
        if matches!(
            category,
            "task" | "withdrawal" | "commission" | "appeal" | "risk"
        ) {
            require(!relation.is_empty(), "请选择本人关联业务")?;
        }
        if category == "appeal" {
            require(
                p["configured"] == true,
                "申诉期限政策尚未配置，请先提交普通任务咨询",
            )?;
            require(
                original["status"] == "rejected",
                "仅对已驳回任务提交审核申诉",
            )?;
            text(i, "evidence")?;
            let at = original["decisions"]
                .as_array()
                .and_then(|d| d.last())
                .and_then(|d| d["at"].as_i64())
                .unwrap_or(0);
            require(
                now - at <= number(&p, "appeal_seconds")?,
                "已超过当前政策申诉期限",
            )?;
            let prior = s
                .cases()
                .iter()
                .filter(|t| {
                    t["kind"] == "ticket"
                        && t["category"] == "appeal"
                        && t["related_id"] == relation
                })
                .count();
            require(
                (prior as i64) < number(&p, "max_appeals")?,
                "已达到申诉次数上限",
            )?;
        }
        let t = json!({"id":id,"kind":"ticket","owner":a,"category":category,"description":i["description"],"related_id":relation,"evidence":i["evidence"],"status":"open","version":1,"created_at":now,"policy":p,"original_reviewer":original["reviewer"],"queue":"support","assigned_to":null,"events":[{"action":action,"actor":a,"at":now}],"replies":[],"notice":"提交申诉不自动改变佣金或线下付款结果"});
        s.cases_mut().push(t.clone());
        return Ok(t);
    }
    let pos = s
        .cases()
        .iter()
        .position(|t| t["id"] == i["id"] && t["kind"] == "ticket")
        .ok_or(Failure("NOT_FOUND", "工单不存在"))?;
    let mut t = s.cases()[pos].clone();
    version(&t, i)?;
    text(i, "reason")?;
    match action {
        "ticket_assign" => {
            require(t["status"] != "closed", "已结案工单不能转派")?;
            let target = text(i, "assigned_to")?;
            s.authorize(target, "support", text(&t, "owner")?)?;
            text(i, "queue")?;
            require(number(i, "due_at")? > now, "请设置未来处理时限")?;
            t["assigned_to"] = i["assigned_to"].clone();
            t["queue"] = i["queue"].clone();
            t["due_at"] = i["due_at"].clone();
            t["status"] = json!("processing");
        }
        "ticket_reply" => {
            require(t["status"] != "closed", "已结案工单请按规则重新打开")?;
            text(i, "body")?;
            require(i["internal"].is_boolean(), "请选择内部备注或用户可见回复")?;
            if t["owner"] == a {
                require(i["internal"] == false, "用户不能写入内部备注")?;
            }
            let attachments = i["attachments"].as_array().cloned().unwrap_or_default();
            require(attachments.len() <= 10, "附件超过服务资源上限")?;
            t["replies"].as_array_mut().unwrap().push(json!({"id":id,"actor":a,"at":now,"body":i["body"],"internal":i["internal"],"attachments":attachments}));
            if t["owner"] != a && t["first_response_at"].is_null() {
                t["first_response_at"] = json!(now);
            }
        }
        "ticket_close" => {
            require(t["owner"] != a, "不可经办结案本人咨询或申诉")?;
            require(t["status"] != "closed", "工单已结案")?;
            require(t["assigned_to"] == a, "仅当前责任人可以结案")?;
            if t["category"] == "appeal" && t["policy"]["independent"] == true {
                require(t["original_reviewer"] != a, "原审核人不能独立复核本申诉")?;
            }
            text(i, "conclusion")?;
            if let Some(adjustment) = i["adjustment_id"].as_str().filter(|x| !x.is_empty()) {
                require(
                    f["adjustments"][adjustment]["owner"] == t["owner"],
                    "关联调整单不存在或归属不符",
                )?;
                t["adjustment_id"] = json!(adjustment);
            }
            t["status"] = json!("closed");
            t["conclusion"] = i["conclusion"].clone();
            t["closed_at"] = json!(now);
        }
        "ticket_reopen" => {
            require(
                t["owner"] == a && t["status"] == "closed",
                "仅本人可以重新打开已结案工单",
            )?;
            require(
                t["policy"]["configured"] == true
                    && now - t["closed_at"].as_i64().unwrap_or(0)
                        <= number(&t["policy"], "reopen_seconds")?,
                "未配置重开期限或已经超过期限，请提交关联新工单",
            )?;
            t["status"] = json!("open");
            t["reopened_at"] = json!(now);
        }
        _ => return Err(Failure("NOT_FOUND", "工单操作不存在")),
    }
    t["version"] = json!(t["version"].as_u64().unwrap_or(0) + 1);
    let event = json!({"action":action,"actor":a,"at":now,"reason":i["reason"],"assigned_to":t["assigned_to"],"queue":t["queue"]});
    t["events"].as_array_mut().unwrap().push(event);
    s.cases_mut()[pos] = t.clone();
    Ok(t)
}
pub fn read<P: DomainPort + ?Sized>(s: &P, a: &str, section: &str, target: &str) -> Result<Value> {
    if section == "support_policy" {
        return Ok(policy(s));
    }
    let mut rows: Vec<Value> = s
        .cases()
        .iter()
        .filter(|t| {
            t["kind"] == "ticket"
                && (t["owner"] == a
                    || s.authorize(a, "support", t["owner"].as_str().unwrap_or(""))
                        .is_ok())
        })
        .cloned()
        .collect();
    if section == "support_statistics" {
        s.authorize(a, "support", target)?;
        let mut groups = std::collections::BTreeMap::new();
        for t in &rows {
            let key = (
                t["owner"].as_str().unwrap_or(""),
                t["category"].as_str().unwrap_or(""),
                t["related_id"].as_str().unwrap_or(""),
            );
            *groups.entry(key).or_insert(0_usize) += 1;
        }
        let repeated = groups.values().filter(|n| **n > 1).count();
        return Ok(
            json!({"repeated_complaint_groups":repeated,"total":rows.len(),"closed":rows.iter().filter(|t|t["status"]=="closed").count(),"ticket_ids":rows.iter().map(|t|&t["id"]).collect::<Vec<_>>(),"response_seconds":rows.iter().filter_map(|t|Some(t["first_response_at"].as_i64()?-t["created_at"].as_i64()?)).collect::<Vec<_>>(),"resolution_seconds":rows.iter().filter_map(|t|Some(t["closed_at"].as_i64()?-t["created_at"].as_i64()?)).collect::<Vec<_>>()}),
        );
    }
    for t in &mut rows {
        if t["owner"] == a {
            if let Some(replies) = t["replies"].as_array_mut() {
                replies.retain(|r| r["internal"] != true);
            }
            t["events"]
                .as_array_mut()
                .unwrap()
                .retain(|e| e["action"] != "ticket_reply");
        }
    }
    Ok(json!(rows))
}
