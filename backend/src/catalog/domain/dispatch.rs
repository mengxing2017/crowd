pub fn is_command(action: &str) -> bool {
    matches!(
        action,
        "category_save"
            | "category_migrate"
            | "product_save"
            | "product_submit"
            | "product_review"
            | "product_offline"
            | "product_archive"
            | "product_rollback"
            | "policy_publish"
            | "touch_bind"
            | "task_claim"
            | "attribution_dispute"
            | "dispute_resolve"
            | "eligibility_verify"
    )
}
pub fn is_read(section: &str) -> bool {
    matches!(
        section,
        "catalog_admin"
            | "attribution_admin"
            | "my_claims"
            | "my_attribution"
            | "catalog_notifications"
            | "attribution_policies"
            | "eligibility"
    )
}
