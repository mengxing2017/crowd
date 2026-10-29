use crate::reporting::south::port::DomainPort;
use serde_json::{Value, json};
pub(super) fn audit_projection(v: &Value) -> Value {
    json!({"id":v["id"],"sequence":v["sequence"],"actor":v["actor"],"action":v["action"],"object":v["object"],"time":v["time"],"request_id":v["request_id"],"result":v["result"],"rule_version":v["rule_version"],"before":v["before"],"after":v["after"]})
}
pub(crate) fn process_exports<P: DomainPort + ?Sized>(s: &mut P, now: i64) -> usize {
    let mut done = 0;
    for job in s
        .data_mut()
        .records
        .values_mut()
        .filter(|v| v["kind"] == "export" && v["status"] == "queued")
    {
        if job["expires_at"].as_i64().unwrap_or(0) <= now {
            job["status"] = json!("expired");
            job["snapshot"] = Value::Null;
        } else {
            job["content"] = job["snapshot"].take();
            job["status"] = json!("ready");
            job["completed_at"] = json!(now);
        }
        job["version"] = json!(2);
        done += 1;
    }
    done
}
