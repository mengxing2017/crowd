//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn tasks(&self) -> &crate::tasks::domain::Tasks;
    fn tasks_mut(&mut self) -> &mut crate::tasks::domain::Tasks;
    fn users(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::identity::south::port::rules::User>;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User>;
    fn sync_commissions(&mut self, actor: &str, now: i64) -> crate::shared::Result<()>;
    fn update_claim_progress(
        &mut self,
        id: &str,
        status: serde_json::Value,
        released: serde_json::Value,
    );
    fn record_catalog_notification(&mut self, event: serde_json::Value);
    fn catalog_claims(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
    fn catalog_eligibility(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
}
