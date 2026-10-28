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
