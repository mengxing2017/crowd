use crate::privacy::south::port::DomainPort;
use crate::shared::{Failure, Result, require};
use serde_json::{Value, json};
pub(crate) fn is_read(s: &str) -> bool {
    matches!(
        s,
        "privacy_policy" | "privacy_requests" | "privacy_copy" | "privacy_repairs"
    )
}
pub(crate) fn read<P: DomainPort + ?Sized>(
    s: &P,
    a: &str,
    section: &str,
    target: &str,
    now: i64,
) -> Result<Value> {
    if section == "privacy_policy" {
        return Ok(super::commands::policy(s)
            .map(|v| json!({"id":v["id"],"version":v["version"],"content":v["content"]}))
            .unwrap_or(Value::Null));
    }
    if section == "privacy_copy" {
        let v = s
            .data()
            .records
            .get(target)
            .ok_or(Failure("NOT_FOUND", "副本不存在"))?;
        require(v["kind"] == "copy" && v["owner"] == a, "仅本人可下载副本")?;
        require(v["expires_at"].as_i64().unwrap_or(0) > now, "副本已过期")?;
        return Ok(v.clone());
    }
    Ok(json!(
        s.data()
            .records
            .values()
            .filter(|v| if section == "privacy_repairs" {
                v["kind"] == "repair"
                    && (s
                        .authorize(a, "verifier", v["owner"].as_str().unwrap_or(""))
                        .is_ok()
                        || s.authorize(a, "settlement_reviewer", v["owner"].as_str().unwrap_or(""))
                            .is_ok())
            } else {
                matches!(v["kind"].as_str(), Some("copy" | "closure"))
                    && (v["owner"] == a
                        || v["kind"] == "closure"
                            && s.authorize(a, "privacy_manager", v["owner"].as_str().unwrap_or(""))
                                .is_ok())
            })
            .map(|v| {
                let mut v = v.clone();
                v.as_object_mut().unwrap().remove("content");
                v
            })
            .collect::<Vec<_>>()
    ))
}
