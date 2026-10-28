use crate::platform::south::port::State;
use crate::withdrawals::south::port::DomainPort;
impl DomainPort for State {
    fn withdrawals(&self) -> &crate::withdrawals::domain::Withdrawals {
        &self.withdrawals
    }
    fn withdrawals_mut(&mut self) -> &mut crate::withdrawals::domain::Withdrawals {
        &mut self.withdrawals
    }
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()> {
        State::authorize(self, actor, role, target)
    }
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User> {
        State::user(self, id)
    }
    fn commission_source_disputed(&self, app: &serde_json::Value) -> bool {
        crate::commissions::south::port::rules::source_disputed(self, app)
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
    fn commission_summary(&self, owner: &str) -> crate::shared::Result<serde_json::Value> {
        crate::commissions::south::port::rules::summary(self, owner)
    }
    fn sync_commissions(&mut self, actor: &str, now: i64) -> crate::shared::Result<()> {
        crate::commissions::south::port::rules::sync_tasks(self, actor, now)
    }
    fn commission_hold_sources(&self, id: &str) -> serde_json::Value {
        crate::commissions::south::port::rules::hold_sources(self, id)
    }
    fn allocate_commissions(
        &self,
        owner: &str,
        n: i64,
    ) -> crate::shared::Result<serde_json::Value> {
        crate::commissions::south::port::rules::allocate(self, owner, n)
    }
    fn validate_alipay(
        &self,
        receipt: &serde_json::Value,
        rule: &serde_json::Value,
    ) -> crate::shared::Result<()> {
        crate::receipts::south::port::rules::validate_alipay(receipt, rule)
    }
}
