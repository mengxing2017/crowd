use crate::platform::south::port::State;
use crate::promotion::south::port::DomainPort;
impl DomainPort for State {
    fn codes(&self) -> &std::collections::BTreeMap<String, crate::promotion::domain::Code> {
        &self.codes
    }
    fn codes_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<String, crate::promotion::domain::Code> {
        &mut self.codes
    }
    fn rules(&self) -> &Vec<serde_json::Value> {
        &self.rules
    }
    #[cfg(test)]
    fn users(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::identity::south::port::rules::User> {
        &self.users
    }
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()> {
        State::authorize(self, actor, role, target)
    }
    #[cfg(test)]
    fn register_user(&mut self, id: &str, agreement: serde_json::Value, source: Option<String>) {
        State::register_user(self, id, agreement, source)
    }
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User> {
        State::user(self, id)
    }
    fn visible_release(&self, id: &str, now: i64) -> crate::shared::Result<serde_json::Value> {
        State::visible_release(self, id, now)
    }
    fn catalog_products(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.catalog.products
    }
}
