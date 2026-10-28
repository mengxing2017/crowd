use serde_json::Value;
pub trait AuditWriter {
    fn log(&mut self, actor: &str, action: &str, target: &str, request: &str, time: &str);
    fn log_failure(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        request: &str,
        time: &str,
        code: &str,
    );
    fn annotate_audit(&mut self, metadata: Value);
}
