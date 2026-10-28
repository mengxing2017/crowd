//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn cases(&self) -> &Vec<serde_json::Value>;
    fn users(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::identity::south::port::rules::User>;
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User>;
    fn append_receipt_event(
        &mut self,
        actor: &str,
        expected_version: u64,
        event: serde_json::Value,
    ) -> crate::shared::Result<u64>;
    fn withdrawals_applications(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
    fn withdrawals_rules(&self) -> &Vec<serde_json::Value>;
}
