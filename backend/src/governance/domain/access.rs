use crate::platform::domain::State;
use crate::shared::Result;
pub(crate) fn access(state: &State, actor: &str) -> Result<()> {
    state.authorize(actor, "rule_manager", "platform")
}
