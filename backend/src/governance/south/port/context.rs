//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn rules(&self) -> &Vec<serde_json::Value>;
    fn rules_mut(&mut self) -> &mut Vec<serde_json::Value>;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
}
