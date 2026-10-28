use crate::platform::south::port::State;
use crate::tasks::south::port::DomainPort;
impl DomainPort for State {
    fn tasks(&self) -> &crate::tasks::domain::Tasks {
        &self.tasks
    }
    fn tasks_mut(&mut self) -> &mut crate::tasks::domain::Tasks {
        &mut self.tasks
    }
    fn users(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::identity::south::port::rules::User> {
        &self.users
    }
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()> {
        State::authorize(self, actor, role, target)
    }
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User> {
        State::user(self, id)
    }
    fn sync_commissions(&mut self, actor: &str, now: i64) -> crate::shared::Result<()> {
        crate::commissions::south::port::rules::sync_tasks(self, actor, now)
    }
    fn update_claim_progress(
        &mut self,
        id: &str,
        status: serde_json::Value,
        released: serde_json::Value,
    ) {
        if let Some(c) = self.catalog.claims.get_mut(id) {
            c["status"] = status;
            c["reservation_released"] = released;
        }
    }
    fn record_catalog_notification(&mut self, mut event: serde_json::Value) {
        self.catalog.revision += 1;
        event["revision"] = serde_json::json!(self.catalog.revision);
        self.catalog.notifications.push(event);
    }
    fn catalog_claims(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.catalog.claims
    }
    fn catalog_eligibility(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.catalog.eligibility
    }
}
