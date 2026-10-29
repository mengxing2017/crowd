use crate::platform::south::port::State;
pub(crate) fn retire_codes(s: &mut State, owner: &str) {
    for c in s.codes.values_mut().filter(|c| c.owner == owner) {
        c.active = false;
        c.version += 1;
    }
}
