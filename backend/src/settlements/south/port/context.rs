//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort:
    crate::commissions::south::port::LedgerMetadata + crate::audit::south::port::AuditWriter
{
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User>;
    fn read_withdrawal(
        &self,
        actor: &str,
        section: &str,
        target: &str,
        now: i64,
    ) -> crate::shared::Result<serde_json::Value>;
    fn commission_summary(&self, owner: &str) -> crate::shared::Result<serde_json::Value>;
    fn commission_source_disputed(&self, app: &serde_json::Value) -> bool;
    fn release_withdrawal(
        &mut self,
        app: &mut serde_json::Value,
        status: &str,
        actor: &str,
        reason: &str,
        now: i64,
    ) -> crate::shared::Result<()>;
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
    fn save_withdrawal_application(&mut self, id: &str, record: serde_json::Value);
    fn save_withdrawal_preview(&mut self, id: &str, record: serde_json::Value);
    fn commissions_entries(&self) -> &Vec<serde_json::Value>;
    fn withdrawals_applications(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
    fn withdrawals_previews(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
}
