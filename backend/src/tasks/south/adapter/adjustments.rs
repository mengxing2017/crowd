use crate::platform::south::port::State;
use crate::shared::{Failure, Result, require};
use serde_json::json;
pub(crate) fn cancel_pending(s: &mut State, id: &str, a: &str, now: i64) -> Result<()> {
    require(
        !s.tasks.confirmations.contains_key(id),
        "已确认任务不能取消待审数字",
    )?;
    let t = s
        .tasks
        .instances
        .get_mut(id)
        .ok_or(Failure("NOT_FOUND", "任务不存在"))?;
    require(
        !matches!(
            t["status"].as_str(),
            Some("invalidated" | "approved" | "disputed")
        ),
        "任务已结束或已有确认",
    )?;
    t["status"] = json!("invalidated");
    t["reservation_released"] = json!(true);
    t["version"] = json!(t["version"].as_u64().unwrap_or(0) + 1);
    t["events"]
        .as_array_mut()
        .unwrap()
        .push(json!({"action":"adjustment_cancel","actor":a,"at":now}));
    if let Some(c) = s.catalog.claims.get_mut(id) {
        c["status"] = json!("invalidated");
        c["reservation_released"] = json!(true);
    }
    Ok(())
}
