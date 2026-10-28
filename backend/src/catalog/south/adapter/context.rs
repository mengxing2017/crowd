use crate::catalog::south::port::DomainPort;
use crate::platform::south::port::State;
impl DomainPort for State {
    fn catalog(&self) -> &crate::catalog::domain::Catalog {
        &self.catalog
    }
    fn catalog_mut(&mut self) -> &mut crate::catalog::domain::Catalog {
        &mut self.catalog
    }
    fn codes(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::promotion::south::port::rules::Code> {
        &self.codes
    }
    fn users(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::identity::south::port::rules::User> {
        &self.users
    }
    fn attach_task(
        &mut self,
        claim: &serde_json::Value,
        snapshot: Option<serde_json::Value>,
        now: i64,
    ) -> crate::shared::Result<()> {
        State::attach_task(self, claim, snapshot, now)
    }
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()> {
        State::authorize(self, actor, role, target)
    }
    fn task_snapshot(
        &self,
        actor: &str,
        task: &serde_json::Value,
        now: i64,
    ) -> crate::shared::Result<Option<serde_json::Value>> {
        State::task_snapshot(self, actor, task, now)
    }
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User> {
        State::user(self, id)
    }
    fn validate_task_reference(
        &self,
        task: &mut serde_json::Value,
        start: i64,
        end: i64,
    ) -> crate::shared::Result<()> {
        State::validate_task_reference(self, task, start, end)
    }
    fn tasks_templates(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.tasks.templates
    }
}
