use crate::shared::{Failure, Result};
pub(crate) fn source_window(source: &str) -> Result<i64> {
    std::env::var(format!(
        "CROWD_TASK_SOURCE_{}_WINDOW_SECONDS",
        source.to_ascii_uppercase()
    ))
    .ok()
    .and_then(|v| v.parse::<i64>().ok())
    .filter(|v| *v > 0)
    .ok_or(Failure("SOURCE_DISABLED", "任务来源尚未配置传输时限"))
}
