use crate::platform::south::port::State;
use crate::support::south::port::DomainPort;
impl DomainPort for State {
    fn authorize(&self, a: &str, r: &str, t: &str) -> crate::shared::Result<()> {
        State::authorize(self, a, r, t)
    }
    fn facts(&self) -> serde_json::Value {
        serde_json::json!({"tasks":self.tasks.instances,"codes":self.codes.iter().map(|(id,c)|(id,serde_json::json!({"owner":c.owner}))).collect::<std::collections::BTreeMap<_,_>>(),"risk":self.risk.records,"adjustments":self.commissions.adjustments})
    }

    fn cases(&self) -> &Vec<serde_json::Value> {
        &self.cases
    }
    fn cases_mut(&mut self) -> &mut Vec<serde_json::Value> {
        &mut self.cases
    }
    fn commissions_entries(&self) -> &Vec<serde_json::Value> {
        &self.commissions.entries
    }
    fn withdrawals_applications(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.withdrawals.applications
    }
}
