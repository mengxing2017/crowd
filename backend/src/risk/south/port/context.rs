use serde_json::Value;
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    #[allow(clippy::too_many_arguments)]
    fn hold_source(
        &mut self,
        a: &str,
        owner: &str,
        source: &str,
        amount: i64,
        id: &str,
        reason: &str,
        now: i64,
    ) -> crate::shared::Result<()>;
    fn release_source(
        &mut self,
        a: &str,
        id: &str,
        reason: &str,
        now: i64,
    ) -> crate::shared::Result<()>;
    fn data(&self) -> &crate::risk::domain::Risk;
    fn data_mut(&mut self) -> &mut crate::risk::domain::Risk;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> crate::shared::Result<()>;
    fn facts(&self) -> Value;
}
