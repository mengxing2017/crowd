use crate::{platform::south::port::State, reporting::south::port::DomainPort};
impl DomainPort for State {
    fn data(&self) -> &crate::reporting::domain::Reporting {
        &self.reporting
    }
    fn data_mut(&mut self) -> &mut crate::reporting::domain::Reporting {
        &mut self.reporting
    }
    fn authorize(&self, a: &str, r: &str, t: &str) -> crate::shared::Result<()> {
        State::authorize(self, a, r, t)
    }
    fn facts(&self) -> serde_json::Value {
        serde_json::json!({"users":self.users.iter().map(|(id,u)|serde_json::json!({"id":id,"source":u.source.as_ref().and_then(|code|self.codes.get(code)).map(|code|&code.owner),"source_product":u.source.as_ref().and_then(|code|self.codes.get(code)).and_then(|code|code.product.as_ref()),"registered_at":u.agreement.get("registered_at").unwrap_or(&u.agreement["time"])})).collect::<Vec<_>>(),"touches":self.catalog.touches,"claims":self.catalog.claims,"tasks":self.tasks.instances,"entries":self.commissions.entries,"applications":self.withdrawals.applications,"audit":self.audit})
    }
}
