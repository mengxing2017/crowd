use serde_json::{Value, json};
pub(crate) fn mask_receipt(v: &Value) -> Value {
    let mut v = v.clone();
    if let Some(s) = v["account"].as_str() {
        let chars: Vec<char> = s.chars().collect();
        let tail: String = chars.iter().rev().take(2).rev().collect();
        v["account"] = json!(format!("***{tail}"));
    }
    v.as_object_mut().unwrap().remove("name");
    v
}

// Defaults are derived for display. Never rewrite an older receipt version.
pub(crate) fn mask_receipts(receipts: &[Value]) -> Vec<Value> {
    receipts
        .iter()
        .map(|r| {
            let mut safe = mask_receipt(r);
            if let Some(method) = r["method"].as_str() {
                let current = receipts.iter().rev().find(|candidate| {
                    candidate["method"] == method
                        && candidate["status"] == "confirmed"
                        && !receipts
                            .iter()
                            .any(|event| event["disables"] == candidate["id"])
                });
                safe["is_default"] =
                    json!(current.is_some_and(|candidate| candidate["id"] == r["id"]));
            }
            safe
        })
        .collect()
}
