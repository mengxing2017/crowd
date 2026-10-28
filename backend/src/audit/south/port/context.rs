//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort {
    fn audit(&self) -> &Vec<serde_json::Value>;
    fn audit_mut(&mut self) -> &mut Vec<serde_json::Value>;
    fn rules(&self) -> &Vec<serde_json::Value>;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
}
