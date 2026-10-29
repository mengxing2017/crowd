use crate::audit::south::port::DomainPort;

use serde_json::{Value, json};
pub trait BehaviorRules: DomainPort {
    fn log(&mut self, actor: &str, action: &str, target: &str, request: &str, time: &str) {
        let published: Vec<Value> = self
            .rules()
            .iter()
            .filter(|r| r["status"] == "published")
            .map(|r| json!({"id":r["id"],"kind":r["kind"],"version":r["version"]}))
            .collect();
        let sequence = self.audit().len() + 1;
        self.audit_mut().push(json!({"id":format!("audit:{sequence}"),"sequence":sequence,"published_rules":published,"actor":actor,"action":action,"object":target,"request_id":request,"time":time,"result":"success","rule_version":"M01-M04-v1"}));
    }
    fn log_failure(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        request: &str,
        time: &str,
        code: &str,
    ) {
        self.log(actor, action, target, request, time);
        if let Some(last) = self.audit_mut().last_mut() {
            last["result"] = json!(code);
        }
    }
}
impl<T: DomainPort + ?Sized> BehaviorRules for T {}
