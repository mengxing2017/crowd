//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn users(&self) -> &std::collections::BTreeMap<String, crate::identity::domain::User>;
    fn users_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<String, crate::identity::domain::User>;
}
