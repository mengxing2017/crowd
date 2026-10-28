use crate::audit::south::port::AuditWriter;
use crate::platform::south::port::State;
use serde_json::Value;
impl AuditWriter for State {
    fn log(&mut self, actor: &str, action: &str, target: &str, request: &str, time: &str) {
        State::log(self, actor, action, target, request, time);
    }
    fn log_failure(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        request: &str,
        time: &str,
        code: &str,
    ) {
        State::log_failure(self, actor, action, target, request, time, code);
    }
    fn annotate_audit(&mut self, metadata: Value) {
        if let (Some(last), Some(metadata)) = (self.audit.last_mut(), metadata.as_object()) {
            for (key, value) in metadata {
                last[key] = value.clone();
            }
        }
    }
}
