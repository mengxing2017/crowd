use crate::shared::Result;
use serde_json::Value;
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn data(&self) -> &crate::reporting::domain::Reporting;
    fn data_mut(&mut self) -> &mut crate::reporting::domain::Reporting;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> Result<()>;
    fn facts(&self) -> Value;
}
