//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn security(&self) -> &crate::identity::domain::Security;
    fn security_mut(&mut self) -> &mut crate::identity::domain::Security;
    fn epoch(&self) -> i64;
    fn users(&self) -> &std::collections::BTreeMap<String, crate::identity::domain::User>;
    fn users_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<String, crate::identity::domain::User>;
}
