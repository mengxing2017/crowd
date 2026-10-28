use crate::identity::south::port::DomainPort;
use crate::platform::south::port::State;
impl DomainPort for State {
    fn users(&self) -> &std::collections::BTreeMap<String, crate::identity::domain::User> {
        &self.users
    }
    fn users_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<String, crate::identity::domain::User> {
        &mut self.users
    }
}
