use crate::platform::south::port::State;
use crate::support::south::port::DomainPort;
impl DomainPort for State {
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
