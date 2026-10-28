use crate::platform::south::port::State;
use crate::receipts::south::port::DomainPort;
impl DomainPort for State {
    fn cases(&self) -> &Vec<serde_json::Value> {
        &self.cases
    }
    fn users(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::identity::south::port::rules::User> {
        &self.users
    }
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User> {
        State::user(self, id)
    }
    fn append_receipt_event(
        &mut self,
        actor: &str,
        expected_version: u64,
        event: serde_json::Value,
    ) -> crate::shared::Result<u64> {
        let u = self
            .users
            .get_mut(actor)
            .ok_or(crate::shared::Failure("SESSION_EXPIRED", "请重新登录"))?;
        crate::shared::check_version(u.version, &serde_json::json!({"version":expected_version}))?;
        u.receipts.push(event);
        u.version += 1;
        Ok(u.version)
    }
    fn withdrawals_applications(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.withdrawals.applications
    }
    fn withdrawals_rules(&self) -> &Vec<serde_json::Value> {
        &self.withdrawals.rules
    }
}
