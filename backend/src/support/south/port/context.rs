//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn authorize(&self, a: &str, r: &str, t: &str) -> crate::shared::Result<()>;
    fn facts(&self) -> serde_json::Value;
    fn cases(&self) -> &Vec<serde_json::Value>;
    fn cases_mut(&mut self) -> &mut Vec<serde_json::Value>;
    fn commissions_entries(&self) -> &Vec<serde_json::Value>;
    fn withdrawals_applications(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
}
