use crate::identity::south::port::DomainPort;
use crate::platform::south::port::State;
impl DomainPort for State {
    fn security(&self) -> &crate::identity::domain::Security {
        &self.security
    }
    fn security_mut(&mut self) -> &mut crate::identity::domain::Security {
        &mut self.security
    }
    fn epoch(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    fn users(&self) -> &std::collections::BTreeMap<String, crate::identity::domain::User> {
        &self.users
    }
    fn users_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<String, crate::identity::domain::User> {
        &mut self.users
    }
}
