use crate::platform::south::port::State;
use crate::shared::Result;
use serde_json::Value;
pub(crate) fn commission_summary(s: &State, actor: &str) -> Result<Value> {
    crate::commissions::south::port::rules::read(s, actor, "commission_summary", actor)
}
