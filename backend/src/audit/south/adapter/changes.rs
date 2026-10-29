use crate::platform::south::port::State;
use serde_json::{Value, json};
fn summary(v: &Value) -> Value {
    json!({"id":v["id"],"version":v["version"],"status":v["status"],"amount_cents":v["amount_cents"],"roles":v["roles"],"scope":v["scope"]})
}
fn object(s: &State, id: &str) -> Value {
    if let Some(u) = s.users.get(id) {
        return json!({"id":id,"version":u.version,"status":u.status,"roles":u.roles,"scope":u.scope});
    }
    for map in [
        &s.catalog.products,
        &s.tasks.instances,
        &s.withdrawals.applications,
        &s.commissions.adjustments,
    ] {
        if let Some(v) = map.get(id) {
            return summary(v);
        }
    }
    Value::Null
}
pub(crate) fn annotate_change(s: &mut State, before: &State, input: &Value, result: &Value) {
    let id = input["user_id"]
        .as_str()
        .or_else(|| input["id"].as_str())
        .or_else(|| result["id"].as_str())
        .unwrap_or("");
    let old = object(before, id);
    let next = object(s, id);
    let value = if next.is_null() {
        summary(result)
    } else {
        next
    };
    if let Some(last) = s
        .audit
        .last_mut()
        .filter(|v| v["request_id"].as_str().is_some())
    {
        last["before"] = old;
        last["after"] = value;
    }
}
