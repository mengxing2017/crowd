use crate::platform::{
    domain::{Result, State},
    north::local::CommandContext,
};
use serde_json::{Value, json};

pub(crate) fn apply(state: &mut State, context: &CommandContext<'_>) -> Result<Value> {
    let CommandContext {
        actor,
        action,
        input,
        generated,
        request,
        now,
        epoch,
    } = *context;
    let result = state.catalog_command(actor, action, input, generated, epoch)?;
    state.log(
        actor,
        action,
        input["id"]
            .as_str()
            .or_else(|| input["user_id"].as_str())
            .or_else(|| input["claim_id"].as_str())
            .or_else(|| result["id"].as_str())
            .unwrap_or(actor),
        request,
        now,
    );
    if let Some(audit) = state.audit.last_mut() {
        audit["rule_version"] = json!("M05-M07-v1");
        audit["result_ref"] = result["id"].clone();
        audit["requirements"] = json!(match action {
            "category_save" | "category_migrate" => vec!["REQ-06-002"],
            "product_save" => vec![
                "REQ-06-001",
                "REQ-06-003",
                "REQ-06-004",
                "REQ-06-005",
                "REQ-06-008",
                "REQ-07-001"
            ],
            "product_submit" => vec!["REQ-07-002", "REQ-07-003"],
            "product_review" => vec!["REQ-06-009", "REQ-07-003", "REQ-07-004", "REQ-07-007"],
            "product_offline" => vec!["REQ-07-005", "REQ-07-006", "REQ-07-010"],
            "product_archive" => vec!["REQ-06-010"],
            "product_rollback" => vec!["REQ-07-008"],
            "policy_publish" => vec!["REQ-05-002", "REQ-05-003"],
            "touch_bind" => vec!["REQ-05-004"],
            "task_claim" => vec!["REQ-05-005", "REQ-05-006", "REQ-05-007", "REQ-06-007"],
            "attribution_dispute" | "dispute_resolve" => vec!["REQ-05-008"],
            "eligibility_verify" => vec!["REQ-06-004"],
            _ => vec![],
        });
        audit["reason"] = input["reason"].clone();
        audit["risk_id"] = input["risk_id"].clone();
        audit["impact"] = input["impact"].clone();
        audit["object_version"] = input["version"].clone();
        audit["release_id"] = input["release_id"].clone();
        audit["policy_version"] = result["policy"]["version"].clone();
    }
    Ok(result)
}
