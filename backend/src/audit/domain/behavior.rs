use crate::platform::domain::State;

use serde_json::{Value, json};
impl State {
    pub fn log(&mut self, actor: &str, action: &str, target: &str, request: &str, time: &str) {
        let published: Vec<Value> = self
            .rules
            .iter()
            .filter(|r| r["status"] == "published")
            .map(|r| json!({"id":r["id"],"kind":r["kind"],"version":r["version"]}))
            .collect();
        self.audit.push(json!({"published_rules":published,"actor":actor,"action":action,"object":target,"request_id":request,"time":time,"result":"success","rule_version":"M01-M04-v1"}));
    }
    pub fn log_failure(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        request: &str,
        time: &str,
        code: &str,
    ) {
        self.log(actor, action, target, request, time);
        if let Some(last) = self.audit.last_mut() {
            last["result"] = json!(code);
        }
    }
}
