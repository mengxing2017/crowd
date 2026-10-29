use crate::shared::Result;
use serde_json::Value;
pub(crate) trait DomainPort: crate::audit::south::port::AuditWriter {
    fn data(&self) -> &crate::privacy::domain::Privacy;
    fn data_mut(&mut self) -> &mut crate::privacy::domain::Privacy;
    fn authorize(&self, actor: &str, role: &str, target: &str) -> Result<()>;
    fn facts(&self) -> Value;
    fn repair(&mut self, a: &str, action: &str, i: &Value, id: &str, now: i64) -> Result<Value>;
    fn retire(&mut self, owner: &str) -> Result<()>;
}
