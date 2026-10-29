use crate::commissions::south::port::DomainPort;
use crate::platform::south::port::State;
impl DomainPort for State {
    fn cancel_task(&mut self, id: &str, a: &str, now: i64) -> crate::shared::Result<()> {
        crate::tasks::south::port::cancel_pending(self, id, a, now)
    }
    fn pause_application(&mut self, id: &str, a: &str, now: i64) -> crate::shared::Result<()> {
        crate::withdrawals::south::port::pause_for_adjustment(self, id, a, now)
    }

    fn commissions(&self) -> &crate::commissions::domain::Commissions {
        &self.commissions
    }
    fn commissions_mut(&mut self) -> &mut crate::commissions::domain::Commissions {
        &mut self.commissions
    }
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()> {
        State::authorize(self, actor, role, target)
    }
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User> {
        State::user(self, id)
    }
    fn tasks_confirmations(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.tasks.confirmations
    }
    fn tasks_instances(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.tasks.instances
    }
    fn withdrawals_applications(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.withdrawals.applications
    }
}
