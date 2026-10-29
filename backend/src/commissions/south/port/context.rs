//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn cancel_task(&mut self, id: &str, a: &str, now: i64) -> crate::shared::Result<()>;
    fn pause_application(&mut self, id: &str, a: &str, now: i64) -> crate::shared::Result<()>;
    fn commissions(&self) -> &crate::commissions::domain::Commissions;
    fn commissions_mut(&mut self) -> &mut crate::commissions::domain::Commissions;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User>;
    fn tasks_confirmations(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
    fn tasks_instances(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
    fn withdrawals_applications(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
}
