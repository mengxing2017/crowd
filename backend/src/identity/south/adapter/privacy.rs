use crate::platform::south::port::State;
use crate::shared::{Failure, Result};
pub(crate) fn retire_profile(s: &mut State, owner: &str) -> Result<()> {
    let u = s
        .users
        .get_mut(owner)
        .ok_or(Failure("NOT_FOUND", "用户不存在"))?;
    u.nickname = "已注销用户".into();
    u.avatar = None;
    u.status = "closed".into();
    u.roles.clear();
    u.scope.clear();
    u.version += 1;
    Ok(())
}
