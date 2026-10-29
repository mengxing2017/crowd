use crate::platform::south::port::State;
pub(crate) fn reconcile_user_notices(s: &mut State, now: i64) -> bool {
    let before = (s.notifications.records.len(), s.risk.records.len());
    crate::risk::south::port::rules::scan(s, now);
    crate::notifications::south::port::rules::sync(s, now);
    before != (s.notifications.records.len(), s.risk.records.len())
}
