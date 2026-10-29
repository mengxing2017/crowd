use crate::privacy::south::port::DomainPort;
use crate::shared::{Failure, Result, number, require, text, version};
use serde_json::{Value, json};
pub(crate) fn is_command(a: &str) -> bool {
    matches!(
        a,
        "privacy_file_cleanup"
            | "privacy_policy_publish"
            | "privacy_copy_request"
            | "privacy_close_request"
            | "privacy_close_review"
            | "privacy_repair_preview"
            | "privacy_repair_review"
    )
}
pub(crate) fn access<P: DomainPort + ?Sized>(
    s: &P,
    a: &str,
    action: &str,
    i: &Value,
) -> Result<()> {
    match action {
        "privacy_policy_publish" => s.authorize(a, "rule_manager", "privacy"),
        "privacy_close_review" => {
            let v = s
                .data()
                .records
                .get(text(i, "id")?)
                .ok_or(Failure("NOT_FOUND", "注销申请不存在"))?;
            s.authorize(a, "privacy_manager", text(v, "owner")?)
        }
        "privacy_repair_preview" => s.authorize(a, "settlement_reviewer", text(i, "owner")?),
        "privacy_repair_review" => {
            let v = s
                .data()
                .records
                .get(text(i, "id")?)
                .ok_or(Failure("NOT_FOUND", "预检批次不存在"))?;
            s.authorize(a, "verifier", text(v, "owner")?)
        }
        _ => Ok(()),
    }
}
pub(crate) fn policy<P: DomainPort + ?Sized>(s: &P) -> Option<&Value> {
    s.data().records.get("policy")
}
pub(crate) fn command<P: DomainPort + ?Sized>(
    s: &mut P,
    a: &str,
    action: &str,
    i: &Value,
    id: &str,
    now: i64,
) -> Result<Value> {
    access(s, a, action, i)?;
    text(i, "reason")?;
    let v = match action {
        "privacy_policy_publish" => {
            let old = policy(s).cloned().unwrap_or(json!({"version":0}));
            version(&old, i)?;
            require(
                i["copy_enabled"].is_boolean() && i["closure_enabled"].is_boolean(),
                "请明确副本与注销政策",
            )?;
            let ttl = number(i, "copy_expires_seconds")?;
            require(ttl > 0, "须明确副本下载有效期")?;
            for k in [
                "wechat_receipt_seconds",
                "task_evidence_seconds",
                "settlement_voucher_seconds",
                "support_attachment_seconds",
            ] {
                require(number(i, k)? >= 0, "须分别明确各用途保留期")?;
            }
            require(
                i["preserve_disputes"] == true && i["retain_financial_history"] == true,
                "必须保留必要数字记录、原申请快照与争议证据",
            )?;
            text(i, "notice")?;
            json!({"id":"policy","kind":"policy","version":old["version"].as_u64().unwrap_or(0)+1,"content":i,"publisher":a,"published_at":now})
        }
        "privacy_copy_request" => {
            let p = policy(s).ok_or(Failure("POLICY_REQUIRED", "本人资料副本政策尚未发布"))?;
            require(p["content"]["copy_enabled"] == true, "本人副本尚未启用")?;
            let f = s.facts();
            let profile = f["users"]
                .get(a)
                .ok_or(Failure("NOT_FOUND", "用户不存在"))?;
            let entries:Vec<Value>=f["entries"].as_array().into_iter().flatten().filter(|v|v["owner"]==a).map(|v|json!({"id":v["id"],"kind":v["kind"],"amount_cents":v["amount_cents"],"currency":v["currency"],"source":v["source"],"at":v["at"]})).collect();
            let applications:Vec<Value>=f["applications"].as_object().into_iter().flat_map(|m|m.values()).filter(|v|v["owner"]==a).map(|v|json!({"id":v["id"],"amount_cents":v["amount_cents"],"status":v["status"],"created_at":v["created_at"],"receipt_method":v["receipt_snapshot"]["method"],"receipt_version":v["receipt_snapshot"]["version"],"paid_at":v["registration"]["paid_at"],"recorded_at":v["registration"]["recorded_at"]})).collect();
            json!({"id":id,"kind":"copy","version":1,"owner":a,"status":"ready","created_at":now,"expires_at":now.checked_add(number(&p["content"],"copy_expires_seconds")?).ok_or(Failure("INVALID_INPUT","有效期超出范围"))?,"policy_version":p["version"],"content":{"profile":profile,"entries":entries,"applications":applications,"notice":"仅本人资料与内部数字记录；不包含内部凭据、他人信息或官方到账证明"}})
        }
        "privacy_close_request" => {
            let p = policy(s).ok_or(Failure("POLICY_REQUIRED", "注销政策尚未发布"))?;
            require(
                p["content"]["closure_enabled"] == true,
                "注销审批政策尚未启用",
            )?;
            require(
                !s.data()
                    .records
                    .values()
                    .any(|v| v["kind"] == "closure" && v["owner"] == a && v["status"] == "pending"),
                "已有待核查注销申请",
            )?;
            json!({"id":id,"kind":"closure","owner":a,"version":1,"status":"pending","reason":i["reason"],"created_at":now,"policy_snapshot":p})
        }
        "privacy_close_review" => {
            let mut v = s
                .data()
                .records
                .get(text(i, "id")?)
                .cloned()
                .ok_or(Failure("NOT_FOUND", "申请不存在"))?;
            version(&v, i)?;
            require(
                v["status"] == "pending" && v["owner"] != a,
                "状态不允许或不能自审注销",
            )?;
            let owner = text(&v, "owner")?.to_owned();
            if i["approve"] == true {
                let f = s.facts();
                require(
                    !f["applications"]
                        .as_object()
                        .into_iter()
                        .flat_map(|m| m.values())
                        .any(|v| {
                            v["owner"] == owner
                                && !matches!(
                                    v["status"].as_str(),
                                    Some("settled" | "cancelled" | "rejected")
                                )
                        }),
                    "尚有未结申请，保留原资料与占用",
                )?;
                require(
                    !f["disputes"]
                        .as_object()
                        .into_iter()
                        .flat_map(|m| m.values())
                        .any(|v| v["owner"] == owner && v["status"] == "open"),
                    "尚有数字争议未结",
                )?;
                require(
                    !f["tasks"]
                        .as_object()
                        .into_iter()
                        .flat_map(|m| m.values())
                        .any(|v| {
                            v["owner"] == owner
                                && !matches!(
                                    v["status"].as_str(),
                                    Some(
                                        "approved"
                                            | "rejected"
                                            | "expired"
                                            | "abandoned"
                                            | "invalidated"
                                    )
                                )
                        }),
                    "尚有任务未结",
                )?;
                require(
                    !f["tickets"].as_array().into_iter().flatten().any(|v| {
                        v["owner"] == owner && v["kind"] == "ticket" && v["status"] != "closed"
                    }),
                    "尚有客服或申诉事项未结",
                )?;
                require(
                    i["retention_confirmed"] == true,
                    "请确认必要数字及结算证据受限保留",
                )?;
                s.retire(&owner)?;
                v["status"] = json!("approved");
            } else {
                v["status"] = json!("rejected");
            }
            v["reviewer"] = json!(a);
            v["review_reason"] = i["reason"].clone();
            v["reviewed_at"] = json!(now);
            v["version"] = json!(v["version"].as_u64().unwrap_or(0) + 1);
            v
        }
        "privacy_repair_preview" => {
            let owner = text(i, "owner")?;
            let f = s.facts();
            for item in i["items"]
                .as_array()
                .ok_or(Failure("INVALID_INPUT", "请选择已有依据的调整单"))?
            {
                let p = f["adjustments"]
                    .get(text(item, "id")?)
                    .ok_or(Failure("NOT_FOUND", "调整不存在"))?;
                require(p["owner"] == owner, "预检不允许混入其他用户")?;
            }
            let batch = s.repair(a, "adjustment_batch_preview", i, id, now)?;
            json!({"id":id,"kind":"repair","version":1,"owner":owner,"creator":a,"status":"previewed","created_at":now,"batch":batch,"reason":i["reason"]})
        }
        "privacy_repair_review" => {
            let mut v = s
                .data()
                .records
                .get(text(i, "id")?)
                .cloned()
                .ok_or(Failure("NOT_FOUND", "预检不存在"))?;
            version(&v, i)?;
            require(
                v["status"] == "previewed" && v["creator"] != a,
                "修复需独立复核且不得重复执行",
            )?;
            require(i["confirmed"] == true, "请确认预检影响清单")?;
            let batch=s.repair(a,"adjustment_batch_review",&json!({"id":v["batch"]["id"],"version":v["batch"]["version"],"reason":i["reason"],"confirmed":true}),id,now)?;
            v["batch"] = batch;
            v["status"] = json!("reviewed");
            v["version"] = json!(2);
            v["reviewer"] = json!(a);
            v
        }
        _ => return Err(Failure("NOT_FOUND", "隐私操作不存在")),
    };
    s.data_mut()
        .history
        .push(json!({"action":action,"actor":a,"at":now,"record":v}));
    s.data_mut()
        .records
        .insert(text(&v, "id")?.into(), v.clone());
    let mut output = v;
    if output["kind"] == "copy" {
        output.as_object_mut().unwrap().remove("content");
    }
    Ok(output)
}
