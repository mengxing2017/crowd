use super::validation::not_found;
use crate::community::domain::{Failure, Result, State, text};
use serde_json::Value;

impl State {
    pub fn catalog_access(&self, actor: &str, action: &str, input: &Value) -> Result<()> {
        let user = self.user(actor)?;
        if user.status != "active" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号当前不可操作"));
        }
        let id = input["id"].as_str().unwrap_or("platform");
        match action {
            "category_save" | "category_migrate" => self.authorize(actor, "operator", "platform"),
            "product_save" => self.authorize(actor, "operator", id),
            "product_submit" | "product_archive" | "product_rollback" => {
                self.authorize(actor, "operator", text(input, "id")?)
            }
            "product_offline" if input["emergency"] == true => {
                self.authorize(actor, "product_emergency", text(input, "id")?)
            }
            "product_offline" => self.authorize(actor, "operator", text(input, "id")?),
            "product_review" => self.authorize(actor, "product_reviewer", text(input, "id")?),
            "policy_publish" => self.authorize(actor, "rule_manager", "platform"),
            "dispute_resolve" => self.authorize(actor, "attribution_reviewer", text(input, "id")?),
            "eligibility_verify" => {
                self.authorize(actor, "identity_reviewer", text(input, "user_id")?)
            }
            "task_claim" => self.authorize(actor, "user", actor),
            "touch_bind" | "attribution_dispute" => self.authorize(actor, "user", actor),
            _ => Err(not_found()),
        }
    }
}
