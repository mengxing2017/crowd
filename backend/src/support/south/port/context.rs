//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn cases(&self) -> &Vec<serde_json::Value>;
    fn cases_mut(&mut self) -> &mut Vec<serde_json::Value>;
    fn commissions_entries(&self) -> &Vec<serde_json::Value>;
    fn withdrawals_applications(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
}
