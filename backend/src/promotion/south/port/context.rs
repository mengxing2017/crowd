//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn codes(&self) -> &std::collections::BTreeMap<String, crate::promotion::domain::Code>;
    fn codes_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<String, crate::promotion::domain::Code>;
    fn rules(&self) -> &Vec<serde_json::Value>;
    #[cfg(test)]
    fn users(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::identity::south::port::rules::User>;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
    #[cfg(test)]
    fn register_user(&mut self, id: &str, agreement: serde_json::Value, source: Option<String>);
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User>;
    fn visible_release(&self, id: &str, now: i64) -> crate::shared::Result<serde_json::Value>;
    fn catalog_products(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
}
