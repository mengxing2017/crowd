use crate::notifications::south::port::DomainPort;
use crate::platform::south::port::State;
impl DomainPort for State {
    fn data(&self) -> &crate::notifications::domain::Notifications {
        &self.notifications
    }
    fn data_mut(&mut self) -> &mut crate::notifications::domain::Notifications {
        &mut self.notifications
    }
    fn authorize(&self, a: &str, r: &str, t: &str) -> crate::shared::Result<()> {
        State::authorize(self, a, r, t)
    }
    fn facts(&self) -> serde_json::Value {
        serde_json::json!({"tasks":self.tasks.instances,"entries":self.commissions.entries,"applications":self.withdrawals.applications,"risk":self.risk.records,"campaigns":self.campaigns.records.values().filter_map(|r|crate::campaigns::south::port::rules::report(self,r).ok().map(|statistics|serde_json::json!({"id":r["id"],"enabled":r["enabled"],"limit_cents":r["limit_cents"],"used_cents":statistics["used_cents"]}))).collect::<Vec<_>>(),"cases":self.cases,"users":self.users.iter().map(|(id,u)|(id.clone(),serde_json::json!({"roles":u.roles,"scope":u.scope}))).collect::<std::collections::BTreeMap<_,_>>()})
    }
}
