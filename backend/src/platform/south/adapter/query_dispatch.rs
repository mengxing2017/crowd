use crate::platform::domain::{Result, State};
use serde_json::Value;

pub(crate) fn read(
    state: &mut State,
    actor: &str,
    section: &str,
    target: Option<&str>,
    request: &str,
    now: &str,
    epoch: i64,
) -> Result<Value> {
    crate::notifications::south::port::rules::sync(state, epoch);
    if crate::campaigns::south::port::rules::is_read(section) {
        return crate::campaigns::south::port::rules::read(
            state,
            actor,
            section,
            target.unwrap_or(actor),
            epoch,
        );
    }
    if crate::notifications::south::port::rules::is_read(section) {
        return crate::notifications::south::port::rules::read(
            state,
            actor,
            section,
            target.unwrap_or(actor),
            epoch,
        );
    }
    if crate::risk::south::port::rules::is_read(section) {
        return crate::risk::south::port::rules::read(
            state,
            actor,
            section,
            target.unwrap_or(actor),
            epoch,
        );
    }
    if crate::support::south::port::rules::tickets::is_read(section) {
        return crate::support::south::port::rules::tickets::read(
            state,
            actor,
            section,
            target.unwrap_or(actor),
        );
    }
    if section == "adjustments" {
        return crate::commissions::south::port::rules::adjustments::read(
            state,
            actor,
            target.unwrap_or(actor),
        );
    }
    if matches!(
        section,
        "commission_source"
            | "commission_summary"
            | "commission_entries"
            | "commission_proposals"
            | "commission_disputes"
    ) {
        let result = crate::commissions::south::port::rules::read(
            state,
            actor,
            section,
            target.unwrap_or(actor),
        );
        if result.is_ok() {
            state.log(actor, section, target.unwrap_or(actor), request, now);
        }
        result
    } else if matches!(
        section,
        "withdrawal_rules" | "my_withdrawals" | "withdrawal_queue" | "withdrawal_detail"
    ) {
        let result = crate::withdrawals::south::port::rules::read(
            state,
            actor,
            section,
            target.unwrap_or(actor),
            epoch,
        );
        if result.is_ok() {
            state.log(actor, section, target.unwrap_or(actor), request, now);
        }
        result
    } else if crate::tasks::south::port::rules::is_read(section) {
        let result = state.tasks_read(actor, section, target.unwrap_or(actor), epoch);
        if result.is_ok() {
            state.log(actor, section, target.unwrap_or(actor), request, now);
        }
        result
    } else if crate::catalog::south::port::rules::is_read(section) {
        let result = state.catalog_read(actor, section, target.unwrap_or(actor), epoch);
        if result.is_ok() {
            state.log(actor, section, target.unwrap_or(actor), request, now);
        }
        result
    } else {
        state.read(actor, section, target.unwrap_or(actor), request, now)
    }
}
