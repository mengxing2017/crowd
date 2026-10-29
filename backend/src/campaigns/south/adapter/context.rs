use crate::campaigns::south::port::DomainPort;
use crate::platform::south::port::State;
impl DomainPort for State {
    fn data(&self) -> &crate::campaigns::domain::Campaigns {
        &self.campaigns
    }
    fn data_mut(&mut self) -> &mut crate::campaigns::domain::Campaigns {
        &mut self.campaigns
    }
    fn authorize(&self, a: &str, r: &str, t: &str) -> crate::shared::Result<()> {
        State::authorize(self, a, r, t)
    }
    fn facts(&self) -> serde_json::Value {
        serde_json::json!({"products":self.catalog.products,"tasks":self.tasks.instances,"claims":self.catalog.claims,"entries":self.commissions.entries,"applications":self.withdrawals.applications,"users":self.users.keys().collect::<Vec<_>>()})
    }
}
