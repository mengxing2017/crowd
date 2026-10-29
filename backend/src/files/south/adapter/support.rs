use crate::platform::south::port::State;
use crate::shared::Result;
pub(crate) fn support_file_owner(s: &State, a: &str, id: &str) -> Result<Option<String>> {
    for t in &s.cases {
        if t["kind"] != "ticket" {
            continue;
        }
        let owner = t["owner"].as_str().unwrap_or("");
        for r in t["replies"].as_array().into_iter().flatten() {
            if r["attachments"]
                .as_array()
                .is_some_and(|files| files.iter().any(|f| f == id))
            {
                if (owner == a && r["internal"] != true) || s.authorize(a, "support", owner).is_ok()
                {
                    return Ok(Some(r["actor"].as_str().unwrap_or("").into()));
                }
                continue;
            }
        }
    }
    Ok(None)
}

pub(crate) fn retain_fingerprint(s: &mut State, id: &str, fingerprint: String) {
    s.risk.file_fingerprints.insert(id.into(), fingerprint);
}
