use crate::reporting::south::port::DomainPort;
use crate::shared::{Failure, Result, require};
use serde_json::{Value, json};
pub(crate) fn is_read(s: &str) -> bool {
    matches!(
        s,
        "reports" | "report_detail" | "export_jobs" | "export_download" | "differences"
    )
}
fn can_report<P: DomainPort + ?Sized>(s: &P, a: &str, v: &Value) -> bool {
    v["creator"] == a
        && super::statistics::allowed(s, a, v["filters"]["product_id"].as_str().unwrap_or(a))
        && v["snapshot"]["products"]
            .as_array()
            .into_iter()
            .flatten()
            .all(|p| super::statistics::allowed(s, a, p["product_id"].as_str().unwrap_or("")))
        && v["snapshot"]["applications"]
            .as_array()
            .into_iter()
            .flatten()
            .all(|p| super::statistics::owner_allowed(s, a, p["owner"].as_str().unwrap_or("")))
}
pub(crate) fn read<P: DomainPort + ?Sized>(
    s: &P,
    a: &str,
    section: &str,
    target: &str,
    now: i64,
) -> Result<Value> {
    if matches!(section, "report_detail" | "export_download") {
        let v = s
            .data()
            .records
            .get(target)
            .ok_or(Failure("NOT_FOUND", "记录不存在"))?;
        if section == "report_detail" {
            require(
                v["kind"] == "report" && can_report(s, a, v),
                "无此报表当前数据范围",
            )?;
            return Ok(v.clone());
        }
        require(v["kind"] == "export" && v["owner"] == a, "无此导出权限")?;
        require(
            v["scope"].as_array().is_some_and(|scope| {
                scope.iter().all(|t| {
                    if v["export_kind"] == "audit" {
                        s.authorize(a, "auditor", t.as_str().unwrap_or("")).is_ok()
                    } else if t["kind"] == "owner" {
                        super::statistics::owner_allowed(s, a, t["id"].as_str().unwrap_or(""))
                    } else {
                        super::statistics::allowed(s, a, t["id"].as_str().unwrap_or(""))
                    }
                })
            }),
            "导出权限已变更",
        )?;
        require(v["status"] == "ready", "导出仍在排队或已失效")?;
        require(
            v["expires_at"].as_i64().unwrap_or(0) > now,
            "下载已过期，请重新申请",
        )?;
        return Ok(
            json!({"id":v["id"],"format":"json","content":v["content"],"expires_at":v["expires_at"]}),
        );
    }
    let rows: Vec<Value> = s
        .data()
        .records
        .values()
        .filter(|v| match section {
            "reports" => v["kind"] == "report" && can_report(s, a, v),
            "export_jobs" => v["kind"] == "export" && v["owner"] == a,
            "differences" => {
                v["kind"] == "difference"
                    && s.authorize(a, "auditor", v["owner"].as_str().unwrap_or(""))
                        .is_ok()
            }
            _ => false,
        })
        .map(|v| {
            let mut v = v.clone();
            if v["kind"] == "export" {
                v.as_object_mut().unwrap().remove("snapshot");
                v.as_object_mut().unwrap().remove("content");
            }
            v
        })
        .collect();
    Ok(json!(rows))
}
