use crate::{platform::south::port::State, privacy::south::port::DomainPort};
impl DomainPort for State {
    fn data(&self) -> &crate::privacy::domain::Privacy {
        &self.privacy
    }
    fn data_mut(&mut self) -> &mut crate::privacy::domain::Privacy {
        &mut self.privacy
    }
    fn authorize(&self, a: &str, r: &str, t: &str) -> crate::shared::Result<()> {
        State::authorize(self, a, r, t)
    }
    fn facts(&self) -> serde_json::Value {
        serde_json::json!({"receipts":self.users.values().map(|u|&u.receipts).collect::<Vec<_>>(),"users":self.users.iter().map(|(id,u)|(id,serde_json::json!({"id":id,"nickname":u.nickname,"avatar":u.avatar,"version":u.version,"status":u.status}))).collect::<std::collections::BTreeMap<_,_>>(),"tasks":self.tasks.instances,"entries":self.commissions.entries,"applications":self.withdrawals.applications,"disputes":self.commissions.disputes,"tickets":self.cases,"adjustments":self.commissions.adjustments})
    }
    fn retire(&mut self, owner: &str) -> crate::shared::Result<()> {
        crate::identity::south::port::retire_profile(self, owner)?;
        crate::promotion::south::port::retire_codes(self, owner);
        Ok(())
    }
    fn repair(
        &mut self,
        a: &str,
        action: &str,
        i: &serde_json::Value,
        id: &str,
        now: i64,
    ) -> crate::shared::Result<serde_json::Value> {
        crate::commissions::south::port::rules::adjustments::command(self, a, action, i, id, now)
    }
}
