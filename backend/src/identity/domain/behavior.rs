use super::User;
use crate::platform::domain::State;
use crate::shared::{Failure, Result};
use serde_json::Value;
impl State {
    pub fn user(&self, id: &str) -> Result<&User> {
        self.users
            .get(id)
            .filter(|u| u.status != "closed")
            .ok_or(Failure("SESSION_EXPIRED", "请重新登录"))
    }
    pub fn authorize(&self, actor: &str, role: &str, target: &str) -> Result<()> {
        let u = self.user(actor)?;
        if u.status == "disabled" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号已停用，请联系客服申诉"));
        }
        if u.roles.iter().any(|r| r == role) && (u.scope.iter().any(|s| s == "*" || s == target)) {
            Ok(())
        } else {
            Err(Failure("FORBIDDEN", "无此操作权限或数据范围"))
        }
    }
    pub fn register_user(&mut self, id: &str, agreement: Value, source: Option<String>) {
        if self.users.contains_key(id) {
            return;
        }
        self.users.insert(
            id.into(),
            User {
                id: id.into(),
                nickname: "新用户".into(),
                avatar: None,
                version: 1,
                status: "active".into(),
                roles: vec!["user".into()],
                scope: vec![id.into()],
                receipts: vec![],
                restrictions: vec![],
                agreement,
                source,
            },
        );
    }
}
