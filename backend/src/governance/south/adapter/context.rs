use crate::governance::south::port::DomainPort;
use crate::platform::south::port::State;
impl DomainPort for State {
    fn rules(&self) -> &Vec<serde_json::Value> {
        &self.rules
    }
    fn rules_mut(&mut self) -> &mut Vec<serde_json::Value> {
        &mut self.rules
    }
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()> {
        State::authorize(self, actor, role, target)
    }
}
