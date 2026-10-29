use crate::notifications::south::port::DomainPort;
use crate::shared::{Failure, Result, number, require, text, version};
use serde_json::{Value, json};
pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "notification_read"
            | "notification_consent"
            | "notification_template"
            | "notification_delivery"
    )
}
pub fn access<P: DomainPort + ?Sized>(s: &P, a: &str, action: &str, i: &Value) -> Result<()> {
    match action {
        "notification_template" => s.authorize(a, "operator", text(i, "id")?),
        "notification_delivery" => {
            let r = s
                .data()
                .records
                .get(text(i, "id")?)
                .ok_or(Failure("NOT_FOUND", "消息不存在"))?;
            s.authorize(a, "operator", text(r, "owner")?)
        }
        "notification_read" => {
            let r = s
                .data()
                .records
                .get(text(i, "id")?)
                .ok_or(Failure("NOT_FOUND", "消息不存在"))?;
            require(r["owner"] == a, "消息不属于本人")
        }
        _ => Ok(()),
    }
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
    let key = if action == "notification_consent" {
        format!("consent:{a}:{}", text(i, "template")?)
    } else {
        text(i, "id")?.into()
    };
    let old = s.data().records.get(&key).cloned();
    let mut row = old.clone().unwrap_or(json!({"id":key,"version":0}));
    version(&row, i)?;
    match action {
        "notification_read" => {
            row["read_at"] = json!(now);
        }
        "notification_consent" => {
            require(i["granted"].is_boolean(), "请选择授权结果")?;
            text(i, "scene")?;
            row["owner"] = json!(a);
            row["kind"] = json!("consent");
            row["template"] = i["template"].clone();
            row["granted"] = i["granted"].clone();
            row["scene"] = i["scene"].clone();
        }
        "notification_template" => {
            text(i, "reason")?;
            text(i, "title")?;
            text(i, "body")?;
            require(i["preview_confirmed"] == true, "请预览模板和跳转目标后确认")?;
            require(
                matches!(
                    i["target"].as_str(),
                    Some("tasks" | "commissions" | "withdrawals" | "support" | "risk")
                ),
                "模板目标必须为受控业务页面",
            )?;
            let vars = i["variables"]
                .as_array()
                .ok_or(Failure("INVALID_INPUT", "请列出模板变量"))?;
            require(
                ["business_id", "status", "notice"].iter().all(|v| {
                    vars.iter().any(|x| x == *v)
                        && i["body"]
                            .as_str()
                            .unwrap_or("")
                            .contains(&format!("{{{v}}}"))
                }),
                "模板须保留业务编号、状态和非真实资金说明变量",
            )?;
            require(
                vars.iter()
                    .all(|v| matches!(v.as_str(), Some("business_id" | "status" | "notice"))),
                "当前模板只支持业务编号、状态和必要说明变量",
            )?;
            let body = text(i, "body")?;
            for v in body.split('{').skip(1) {
                let name = v.split('}').next().unwrap_or("");
                require(vars.iter().any(|x| x == name), "模板包含未声明变量")?;
            }
            if !i["reminder_seconds"].is_null() {
                require(number(i, "reminder_seconds")? > 0, "提醒窗口须为正整数秒")?;
            }
            if !i["max_attempts"].is_null() || !i["retry_seconds"].is_null() {
                require(
                    number(i, "max_attempts")? > 0 && number(i, "retry_seconds")? > 0,
                    "外部通知须显式配置有限重试策略",
                )?;
            }
            row = json!({"id":key,"version":row["version"],"kind":"template","title":i["title"],"body":i["body"],"variables":vars,"target":i["target"],"reminder_seconds":i["reminder_seconds"],"max_attempts":i["max_attempts"],"retry_seconds":i["retry_seconds"],"status":"published","actor":a});
        }
        "notification_delivery" => {
            // Channel results must come from a configured adapter, never from browser-reported success.
            return Err(Failure(
                "CHANNEL_UNAVAILABLE",
                "尚未配置外部订阅渠道，站内消息仍保留；不能手工伪造发送结果",
            ));
        }
        _ => return Err(Failure("NOT_FOUND", "消息操作不存在")),
    }
    row["version"] = json!(row["version"].as_u64().unwrap_or(0) + 1);
    row["updated_at"] = json!(now);
    s.data_mut()
        .history
        .push(json!({"id":id,"actor":a,"action":action,"at":now,"before":old,"after":row}));
    s.data_mut().records.insert(key, row.clone());
    Ok(row)
}
#[allow(clippy::too_many_arguments)]
fn emit<P: DomainPort + ?Sized>(
    s: &mut P,
    owner: &str,
    key: &str,
    target: &str,
    business: &str,
    status: &str,
    reason: Value,
    now: i64,
) {
    let template = s
        .data()
        .records
        .values()
        .filter(|r| r["kind"] == "template" && r["target"] == target)
        .max_by_key(|r| r["updated_at"].as_i64().unwrap_or(0))
        .cloned();
    let template_id = template
        .as_ref()
        .and_then(|t| t["id"].as_str())
        .unwrap_or("builtin");
    let template_version = template
        .as_ref()
        .and_then(|t| t["version"].as_u64())
        .unwrap_or(1);
    let id = format!("message:{owner}:{key}:{template_id}:{template_version}");
    let notice =
        "平台业务状态通知，佣金仅为标记数字；已登记结算不代表官方到账证明，可通过客服反馈异议";
    let body = template
        .as_ref()
        .and_then(|t| t["body"].as_str())
        .unwrap_or("{business_id}：{status}。{notice}")
        .replace("{business_id}", business)
        .replace("{status}", status)
        .replace("{notice}", notice);
    s.data_mut().records.entry(id.clone()).or_insert(json!({"id":id,"owner":owner,"kind":"message","version":1,"template_version":template_version,"template_id":template_id,"template_snapshot":template,"body":body,"target":target,"business_id":business,"status":status,"reason":reason,"created_at":now,"read_at":null,"delivery":"in_app","external_status":"not_configured","notice":notice}));
}
/// Materialize durable station messages with event + template version + recipient uniqueness.
pub fn sync<P: DomainPort + ?Sized>(s: &mut P, now: i64) {
    let f = s.facts();
    for t in f["tasks"].as_object().into_iter().flat_map(|m| m.values()) {
        let id = t["id"].as_str().unwrap_or("");
        let owner = t["owner"].as_str().unwrap_or("");
        let status = t["status"].as_str().unwrap_or("");
        emit(
            s,
            owner,
            &format!("task:{id}:{}", t["version"]),
            "tasks",
            id,
            status,
            t["decisions"]
                .as_array()
                .and_then(|v| v.last())
                .map(|v| v["reason"].clone())
                .unwrap_or(Value::Null),
            now,
        );
        let window = s
            .data()
            .records
            .values()
            .filter(|v| v["kind"] == "template" && v["target"] == "tasks")
            .filter_map(|v| v["reminder_seconds"].as_i64())
            .max();
        if let Some(window) = window
            && matches!(status, "claimed" | "in_progress" | "supplement")
            && let Some(end) = t["submit_until"].as_i64()
            && end > now
            && end - now <= window
        {
            emit(
                s,
                owner,
                &format!("deadline:{id}:{end}"),
                "tasks",
                id,
                "deadline_reminder",
                json!({"deadline":end,"impact":"未按时完成将按领取快照规则结束"}),
                now,
            );
        }
    }
    for e in f["entries"].as_array().into_iter().flatten() {
        if !matches!(
            e["kind"].as_str(),
            Some("pending" | "hold" | "release" | "settled")
        ) {
            let owner = e["owner"].as_str().unwrap_or("");
            let id = e["id"].as_str().unwrap_or("");
            emit(
                s,
                owner,
                &format!("commission:{id}"),
                "commissions",
                e["source"].as_str().unwrap_or(""),
                e["kind"].as_str().unwrap_or(""),
                json!({"amount_cents":e["amount_cents"]}),
                now,
            );
        }
    }
    for app in f["applications"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.values())
    {
        let id = app["id"].as_str().unwrap_or("");
        emit(
            s,
            app["owner"].as_str().unwrap_or(""),
            &format!("withdrawal:{id}:{}", app["version"]),
            "withdrawals",
            id,
            app["status"].as_str().unwrap_or(""),
            Value::Null,
            now,
        );
    }
    for case in f["cases"].as_array().into_iter().flatten() {
        let id = case["id"].as_str().unwrap_or("");
        emit(
            s,
            case["owner"].as_str().unwrap_or(""),
            &format!("support:{id}:{}", case["version"]),
            "support",
            id,
            case["status"].as_str().unwrap_or(""),
            Value::Null,
            now,
        );
    }
    for case in f["risk"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.values())
        .filter(|r| r["kind"] == "case")
    {
        let id = case["id"].as_str().unwrap_or("");
        emit(
            s,
            case["owner"].as_str().unwrap_or(""),
            &format!("risk:{id}:{}", case["version"]),
            "risk",
            id,
            case["status"].as_str().unwrap_or(""),
            case["resolution"].clone(),
            now,
        );
    }
}
