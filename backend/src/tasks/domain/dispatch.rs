pub fn is_command(a: &str) -> bool {
    matches!(
        a,
        "task_source_configure"
            | "price_save"
            | "price_review"
            | "template_save"
            | "template_review"
            | "template_disable"
            | "task_start"
            | "task_step"
            | "task_abandon"
            | "task_pause"
            | "task_resume"
            | "task_expire"
            | "evidence_discard"
            | "evidence_save"
            | "evidence_preview"
            | "evidence_submit"
            | "evidence_withdraw"
            | "review_claim"
            | "review_decide"
            | "review_verify"
            | "review_batch_preview"
            | "review_batch"
            | "task_repair_propose"
            | "task_repair_review"
            | "review_revoke_propose"
            | "review_revoke_review"
            | "quality_inspect"
            | "evidence_reveal"
    )
}
pub fn is_read(s: &str) -> bool {
    matches!(
        s,
        "task_rules" | "my_tasks" | "task_queue" | "task_detail" | "task_changes"
    )
}
