use serde_json::Value;
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn data(&self) -> &crate::notifications::domain::Notifications;
    fn data_mut(&mut self) -> &mut crate::notifications::domain::Notifications;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
    fn facts(&self) -> Value;
}
