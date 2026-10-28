use crate::platform::south::port::State;
use crate::settlements::south::port::DomainPort;
impl DomainPort for State {
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()> {
        State::authorize(self, actor, role, target)
    }
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User> {
        State::user(self, id)
    }
    fn read_withdrawal(
        &self,
        actor: &str,
        section: &str,
        target: &str,
        now: i64,
    ) -> crate::shared::Result<serde_json::Value> {
        crate::withdrawals::south::port::rules::read(self, actor, section, target, now)
    }
    fn commission_summary(&self, owner: &str) -> crate::shared::Result<serde_json::Value> {
        crate::commissions::south::port::rules::summary(self, owner)
    }
    fn commission_source_disputed(&self, app: &serde_json::Value) -> bool {
        crate::commissions::south::port::rules::source_disputed(self, app)
    }
    fn release_withdrawal(
        &mut self,
        app: &mut serde_json::Value,
        status: &str,
        actor: &str,
        reason: &str,
        now: i64,
    ) -> crate::shared::Result<()> {
        crate::withdrawals::south::port::rules::release(self, app, status, actor, reason, now)
    }
    fn ledger_append(
        &mut self,
        key: &str,
        owner: &str,
        kind: &str,
        amount: i64,
        source: &str,
        actor: &str,
        now: i64,
        detail: serde_json::Value,
    ) -> crate::shared::Result<serde_json::Value> {
        crate::commissions::south::port::rules::append(
            self, key, owner, kind, amount, source, actor, now, detail,
        )
    }
    fn save_withdrawal_application(&mut self, id: &str, record: serde_json::Value) {
        self.withdrawals.applications.insert(id.into(), record);
    }
    fn save_withdrawal_preview(&mut self, id: &str, record: serde_json::Value) {
        self.withdrawals.previews.insert(id.into(), record);
    }
    fn commissions_entries(&self) -> &Vec<serde_json::Value> {
        &self.commissions.entries
    }
    fn withdrawals_applications(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.withdrawals.applications
    }
    fn withdrawals_previews(&self) -> &std::collections::BTreeMap<String, serde_json::Value> {
        &self.withdrawals.previews
    }
}
