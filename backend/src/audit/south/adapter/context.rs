use crate::audit::south::port::DomainPort;
use crate::platform::south::port::State;
impl DomainPort for State {
    fn audit(&self) -> &Vec<serde_json::Value> {
        &self.audit
    }
    fn audit_mut(&mut self) -> &mut Vec<serde_json::Value> {
        &mut self.audit
    }
    fn rules(&self) -> &Vec<serde_json::Value> {
        &self.rules
    }
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()> {
        State::authorize(self, actor, role, target)
    }
}
