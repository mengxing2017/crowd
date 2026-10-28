//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort:
    crate::commissions::south::port::LedgerMetadata + crate::audit::south::port::AuditWriter
{
    fn withdrawals(&self) -> &crate::withdrawals::domain::Withdrawals;
    fn withdrawals_mut(&mut self) -> &mut crate::withdrawals::domain::Withdrawals;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User>;
    fn commission_source_disputed(&self, app: &serde_json::Value) -> bool;
    #[allow(clippy::too_many_arguments)]
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
    ) -> crate::shared::Result<serde_json::Value>;
    fn commission_summary(&self, owner: &str) -> crate::shared::Result<serde_json::Value>;
    fn sync_commissions(&mut self, actor: &str, now: i64) -> crate::shared::Result<()>;
    fn commission_hold_sources(&self, id: &str) -> serde_json::Value;
    fn allocate_commissions(&self, owner: &str, n: i64)
    -> crate::shared::Result<serde_json::Value>;
    fn validate_alipay(
        &self,
        receipt: &serde_json::Value,
        rule: &serde_json::Value,
    ) -> crate::shared::Result<()>;
}
