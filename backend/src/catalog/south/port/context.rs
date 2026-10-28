//! Consumer-owned transaction interfaces. No global state is exposed.
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn catalog(&self) -> &crate::catalog::domain::Catalog;
    fn catalog_mut(&mut self) -> &mut crate::catalog::domain::Catalog;
    fn codes(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::promotion::south::port::rules::Code>;
    fn users(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::identity::south::port::rules::User>;
    fn attach_task(
        &mut self,
        claim: &serde_json::Value,
        snapshot: Option<serde_json::Value>,
        now: i64,
    ) -> crate::shared::Result<()>;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
    fn task_snapshot(
        &self,
        actor: &str,
        task: &serde_json::Value,
        now: i64,
    ) -> crate::shared::Result<Option<serde_json::Value>>;
    fn user(&self, id: &str) -> crate::shared::Result<&crate::identity::south::port::rules::User>;
    fn validate_task_reference(
        &self,
        task: &mut serde_json::Value,
        start: i64,
        end: i64,
    ) -> crate::shared::Result<()>;
    fn tasks_templates(&self) -> &std::collections::BTreeMap<String, serde_json::Value>;
}
