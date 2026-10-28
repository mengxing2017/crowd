use crate::platform::domain::{Result, State};
use serde_json::Value;

pub(super) fn read(
    state: &mut State,
    actor: &str,
    section: &str,
    target: Option<&str>,
    request: &str,
    now: &str,
    epoch: i64,
) -> Result<Value> {
    if matches!(
        section,
        "commission_source"
            | "commission_summary"
            | "commission_entries"
            | "commission_proposals"
            | "commission_disputes"
    ) {
        let result =
            crate::commissions::domain::read(state, actor, section, target.unwrap_or(actor));
        if result.is_ok() {
            state.log(actor, section, target.unwrap_or(actor), request, now);
        }
        result
    } else if matches!(
        section,
        "withdrawal_rules" | "my_withdrawals" | "withdrawal_queue" | "withdrawal_detail"
    ) {
        let result =
            crate::withdrawals::domain::read(state, actor, section, target.unwrap_or(actor), epoch);
        if result.is_ok() {
            state.log(actor, section, target.unwrap_or(actor), request, now);
        }
        result
    } else if crate::tasks::domain::is_read(section) {
        let result = state.tasks_read(actor, section, target.unwrap_or(actor), epoch);
        if result.is_ok() {
            state.log(actor, section, target.unwrap_or(actor), request, now);
        }
        result
    } else if crate::catalog::domain::is_read(section) {
        let result = state.catalog_read(actor, section, target.unwrap_or(actor), epoch);
        if result.is_ok() {
            state.log(actor, section, target.unwrap_or(actor), request, now);
        }
        result
    } else {
        state.read(actor, section, target.unwrap_or(actor), request, now)
    }
}
