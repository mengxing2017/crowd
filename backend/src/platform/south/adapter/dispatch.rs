use super::state::State;
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
            "me" | "users" => {
                crate::identity::south::port::rules::read(self, actor, section, target)
            }
            "codes" | "managed_codes" => {
                crate::promotion::south::port::rules::read(self, actor, section, target)
            }
            "rules" => crate::governance::south::port::rules::read(self, actor, section, target),
            "audit" => crate::audit::south::port::rules::read(self, actor, section, target),
            "cases" => crate::support::south::port::rules::read(self, actor, section, target),
            "receipts" => crate::receipts::south::port::rules::read(self, actor, section, target),
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
        if crate::commissions::south::port::rules::is_command(action) {
            return crate::commissions::south::port::rules::access(self, actor, action, input);
        }
        if crate::withdrawals::south::port::rules::is_command(action) {
            return crate::withdrawals::south::port::rules::access(self, actor, action, input);
        }
        if crate::settlements::south::port::rules::is_command(action) {
            return crate::settlements::south::port::rules::access(self, actor, action, input);
        }
        if crate::tasks::south::port::rules::is_command(action) {
            return self.tasks_access(actor, action, input);
        }
        if crate::catalog::south::port::rules::is_command(action) {
            return self.catalog_access(actor, action, input);
        }
        crate::identity::south::port::rules::validate_input(action, input)?;
        crate::receipts::south::port::rules::validate_input(action, input)?;
        let user = self.user(actor)?;
        if user.status == "disabled" && action != "logout" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号已停用，请联系客服申诉"));
        }
        match action {
            "rule_save" | "rule_publish" | "rule_disable" => {
                crate::governance::south::port::rules::access(self, actor)
            }
            "access" | "restriction" => {
                crate::identity::south::port::rules::access(self, actor, action, input)
            }
            "code_rotate" => {
                crate::promotion::south::port::rules::access(self, actor, action, input)
            }
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
            "profile" | "access" | "restriction" | "phone" => {
                crate::identity::south::port::rules::command(
                    self, actor, action, input, generated, request, time,
                )?
            }
            "receipt" | "receipt_disable" => crate::receipts::south::port::rules::command(
                self, actor, action, input, generated, request, time,
            )?,
            "code_create" | "code_rotate" => crate::promotion::south::port::rules::command(
                self, actor, action, input, generated, request, time,
            )?,
            "rule_save" | "rule_publish" | "rule_disable" => {
                crate::governance::south::port::rules::command(
                    self, actor, action, input, generated, request, time,
                )?
            }
            "correction" | "closure" | "merge" => crate::support::south::port::rules::command(
                self, actor, action, input, generated, request, time,
            )?,
            _ => return Err(Failure("NOT_FOUND", "操作不存在")),
        };
        Ok(result)
    }
}
