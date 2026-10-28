use crate::platform::south::port::State;
use crate::shared::Result;
pub(crate) use crate::tasks::south::port::services::ReadEvidenceFile;
pub(crate) fn validate_upload(s: &State, actor: &str, purpose: &str, size: usize) -> Result<()> {
    crate::tasks::south::port::rules::validate_upload(s, actor, purpose, size)
}
pub(crate) fn register_upload(
    s: &mut State,
    actor: &str,
    task: Option<&str>,
    id: &str,
    size: usize,
) {
    crate::tasks::south::port::rules::register_upload(s, actor, task, id, size)
}
