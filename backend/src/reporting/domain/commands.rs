use super::statistics::{allowed, snapshot};
use crate::reporting::south::port::DomainPort;
use crate::shared::{Failure, Result, number, require, text, version};
use serde_json::{Value, json};
pub(crate) fn is_command(a: &str) -> bool {
    matches!(
        a,
        "report_create"
            | "export_request"
            | "reconciliation_run"
            | "difference_assign"
            | "difference_close"
            | "voucher_check"
    )
}
pub(crate) fn access<P: DomainPort + ?Sized>(
    s: &P,
    a: &str,
    action: &str,
    i: &Value,
) -> Result<()> {
    if matches!(action, "report_create" | "export_request") {
        let t = i["product_id"].as_str().unwrap_or(a);
        require(allowed(s, a, t), "无报表对象数据范围")
    } else {
        let owner = if action == "reconciliation_run" {
            text(i, "owner")?
        } else {
            s.data()
                .records
                .get(text(i, "id")?)
                .and_then(|v| v["owner"].as_str())
                .ok_or(Failure("NOT_FOUND", "差异或申请不存在"))?
        };
        s.authorize(a, "auditor", owner)
    }
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
    let value = match action {
        "report_create" => {
            let report = snapshot(s, a, i, now)?;
            json!({"id":id,"kind":"report","version":1,"creator":a,"created_at":now,"filters":i,"snapshot":report})
        }
        "export_request" => {
            let ttl = number(i, "expires_seconds")?;
            require(ttl > 0, "必须明确导出下载有效期")?;
            let kind = i["export_kind"].as_str().unwrap_or("report");
            require(matches!(kind, "report" | "audit"), "不支持此类导出")?;
            let (snapshot, scope) = if kind == "audit" {
                let target = text(i, "target")?;
                s.authorize(a, "auditor", target)?;
                let from = text(i, "from_time")?;
                let until = text(i, "until_time")?;
                require(from <= until, "审计时间范围错误")?;
                let f = s.facts();
                let rows: Vec<Value> = f["audit"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|v| {
                        (target == "*" || v["object"] == target)
                            && v["time"].as_str().is_some_and(|t| t >= from && t <= until)
                            && i["actor"].as_str().is_none_or(|x| v["actor"] == x)
                    })
                    .map(super::exports::audit_projection)
                    .collect();
                (json!(rows), json!([target]))
            } else {
                let data = snapshot(s, a, i, now)?;
                let mut scope: Vec<Value> = data["products"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|v| json!({"kind":"product","id":v["product_id"]}))
                    .collect();
                scope.push(json!({"kind":"product","id":i["product_id"].as_str().unwrap_or(a)}));
                scope.extend(
                    data["applications"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|v| json!({"kind":"owner","id":v["owner"]})),
                );
                (data, json!(scope))
            };
            json!({"id":id,"kind":"export","version":1,"owner":a,"status":"queued","created_at":now,"expires_at":now.checked_add(ttl).ok_or(Failure("INVALID_INPUT","有效期超出范围"))?,"scope":scope,"export_kind":kind,"snapshot":snapshot,"purpose":i["reason"]})
        }
        "reconciliation_run" => {
            let owner = text(i, "owner")?;
            let f = s.facts();
            let mut keys = std::collections::BTreeSet::new();
            let mut duplicate = vec![];
            let mut totals = std::collections::BTreeMap::<String, i128>::new();
            for e in f["entries"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|e| e["owner"] == owner)
            {
                if !keys.insert(e["key"].as_str().unwrap_or("")) {
                    duplicate.push(e["id"].clone());
                }
                *totals
                    .entry(e["kind"].as_str().unwrap_or("").into())
                    .or_default() += e["amount_cents"].as_i64().unwrap_or(0) as i128;
            }
            let n = |k: &str| totals.get(k).copied().unwrap_or(0);
            let net = n("confirmed") + n("increase") - n("decrease");
            let held = n("hold") - n("release") - n("settled");
            let disputed = n("dispute_hold") - n("dispute_release");
            let apps: Vec<&Value> = f["applications"]
                .as_object()
                .into_iter()
                .flat_map(|m| m.values())
                .filter(|v| v["owner"] == owner)
                .collect();
            let expected: i128 = apps
                .iter()
                .filter(|v| {
                    !matches!(
                        v["status"].as_str(),
                        Some("cancelled" | "rejected" | "settled")
                    )
                })
                .map(|v| v["amount_cents"].as_i64().unwrap_or(0) as i128)
                .sum();
            let settled: i128 = apps
                .iter()
                .filter(|v| v["status"] == "settled")
                .map(|v| v["amount_cents"].as_i64().unwrap_or(0) as i128)
                .sum();
            let consistent = duplicate.is_empty()
                && held == expected
                && n("settled") == settled
                && net >= held + n("settled") + disputed
                && held >= 0
                && disputed >= 0;
            for v in [net, held, expected, settled, disputed] {
                require(
                    v.abs() <= crate::shared::MAX as i128,
                    "核对数字超出精度范围",
                )?;
            }
            json!({"id":id,"kind":"difference","category":"digital","version":1,"owner":owner,"status":if consistent{"matched"}else{"open"},"created_at":now,"watermark":now,"duplicate_entries":duplicate,"net_cents":net as i64,"held_cents":held as i64,"expected_held_cents":expected as i64,"settled_cents":settled as i64,"disputed_cents":disputed as i64,"reason":i["reason"],"history":[]})
        }
        "voucher_check" => {
            let old = s
                .data()
                .records
                .get(text(i, "id")?)
                .cloned()
                .ok_or(Failure("NOT_FOUND", "差异不存在"))?;
            version(&old, i)?;
            let owner = text(&old, "owner")?;
            let f = s.facts();
            let app = f["applications"]
                .get(text(i, "application_id")?)
                .ok_or(Failure("NOT_FOUND", "申请不存在"))?;
            require(
                app["owner"] == owner && !app["registration"].is_null(),
                "申请归属不符或尚无人工登记",
            )?;
            require(
                i["checked_amount"] == true
                    && i["checked_method"] == true
                    && i["checked_snapshot"] == true,
                "请人工核对金额、方式和原收款快照",
            )?;
            let mut v = old;
            v["category"] = json!(if i["matched"] == true {
                "manual_voucher"
            } else {
                "offline_dispute"
            });
            v["voucher_check"] = json!({"application_id":app["id"],"registration_id":app["registration"]["id"],"matched":i["matched"],"reviewer":a,"at":now,"basis":i["reason"],"official_verified":false});
            v["status"] = json!(if i["matched"] == true {
                "matched"
            } else {
                "open"
            });
            v["version"] = json!(v["version"].as_u64().unwrap_or(0) + 1);
            v
        }
        "difference_assign" | "difference_close" => {
            let mut v = s
                .data()
                .records
                .get(text(i, "id")?)
                .cloned()
                .ok_or(Failure("NOT_FOUND", "差异不存在"))?;
            version(&v, i)?;
            require(v["status"] == "open", "差异不在待处理状态")?;
            if action == "difference_assign" {
                let target = text(i, "assignee")?;
                s.authorize(target, "auditor", text(&v, "owner")?)?;
                v["assignee"] = json!(target);
            } else {
                require(v["assignee"] == a, "仅当前责任人可结案")?;
                text(i, "conclusion")?;
                v["status"] = json!("closed");
                v["conclusion"] = i["conclusion"].clone();
                v["correction_ref"] = i["correction_ref"].clone();
            }
            v["history"]
                .as_array_mut()
                .ok_or(Failure("INVALID_INPUT", "历史格式错误"))?
                .push(json!({"actor":a,"action":action,"at":now,"reason":i["reason"]}));
            v["version"] = json!(v["version"].as_u64().unwrap_or(0) + 1);
            v
        }
        _ => return Err(Failure("NOT_FOUND", "报表操作不存在")),
    };
    s.data_mut()
        .records
        .insert(text(&value, "id")?.into(), value.clone());
    s.data_mut().history.push(
        json!({"id":value["id"],"action":action,"actor":a,"at":now,"version":value["version"]}),
    );
    let mut output = value;
    if output["kind"] == "export" {
        output.as_object_mut().unwrap().remove("snapshot");
    }
    Ok(output)
}
