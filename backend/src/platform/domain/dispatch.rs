use crate::platform::domain::State;
use crate::shared::{Failure, Result};
use serde_json::Value;
impl State {
    pub fn read(
        &mut self,
        actor: &str,
        section: &str,
        target: &str,
        request: &str,
        time: &str,
    ) -> Result<Value> {
        self.user(actor)?;
        match section {
            "me" | "users" => crate::identity::domain::read(self, actor, section, target),
            "codes" | "managed_codes" => {
                crate::promotion::domain::read(self, actor, section, target)
            }
            "rules" => crate::governance::domain::read(self, actor, section, target),
            "audit" => crate::audit::domain::read(self, actor, section, target),
            "cases" => crate::support::domain::read(self, actor, section, target),
            "receipts" => crate::receipts::domain::read(self, actor, section, target),
            "command_result" => {
                let previous =
                    self.idempotency
                        .get(&format!("{actor}:{target}"))
                        .ok_or(Failure(
                            "RESULT_NOT_FOUND",
                            "未查询到已提交结果，请使用原请求键重试",
                        ))?;
                self.check_command_access(
                    actor,
                    previous["action"].as_str().unwrap_or(""),
                    &previous["input"],
                )?;
                Ok(previous["result"].clone())
            }
            _ => Err(Failure("NOT_FOUND", "接口不存在")),
        }
        .inspect(|_| {
            self.log(actor, section, target, request, time);
        })
    }
    pub fn check_command_access(&self, actor: &str, action: &str, input: &Value) -> Result<()> {
        if crate::commissions::domain::is_command(action) {
            return crate::commissions::domain::access(self, actor, action, input);
        }
        if crate::withdrawals::domain::is_command(action) {
            return crate::withdrawals::domain::access(self, actor, action, input);
        }
        if crate::settlements::domain::is_command(action) {
            return crate::settlements::domain::access(self, actor, action, input);
        }
        if crate::tasks::domain::is_command(action) {
            return self.tasks_access(actor, action, input);
        }
        if crate::catalog::domain::is_command(action) {
            return self.catalog_access(actor, action, input);
        }
        crate::identity::domain::validate_input(action, input)?;
        crate::receipts::domain::validate_input(action, input)?;
        let user = self.user(actor)?;
        if user.status == "disabled" && action != "logout" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号已停用，请联系客服申诉"));
        }
        match action {
            "rule_save" | "rule_publish" | "rule_disable" => {
                crate::governance::domain::access(self, actor)
            }
            "access" | "restriction" => crate::identity::domain::access(self, actor, action, input),
            "code_rotate" => crate::promotion::domain::access(self, actor, action, input),
            _ => Ok(()),
        }
    }
    pub fn command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        request: &str,
        time: &str,
    ) -> Result<Value> {
        let u = self.user(actor)?;
        if u.status == "disabled" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号已停用，请联系客服申诉"));
        }
        let result = match action {
            "profile" | "access" | "restriction" | "phone" => crate::identity::domain::command(
                self, actor, action, input, generated, request, time,
            )?,
            "receipt" | "receipt_disable" => crate::receipts::domain::command(
                self, actor, action, input, generated, request, time,
            )?,
            "code_create" | "code_rotate" => crate::promotion::domain::command(
                self, actor, action, input, generated, request, time,
            )?,
            "rule_save" | "rule_publish" | "rule_disable" => crate::governance::domain::command(
                self, actor, action, input, generated, request, time,
            )?,
            "correction" | "closure" | "merge" => crate::support::domain::command(
                self, actor, action, input, generated, request, time,
            )?,
            _ => return Err(Failure("NOT_FOUND", "操作不存在")),
        };
        Ok(result)
    }
}
