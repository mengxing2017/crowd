use crate::files::south::port::FileAccess;
use crate::{
    files::south::port::{AuditWriter, Repository},
    shared::text,
    shared::{Failure, Result},
};
use serde_json::json;
#[derive(Clone)]
pub struct ReadSensitiveFile {
    repo: Repository,
}
impl ReadSensitiveFile {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
    pub async fn execute(
        &self,
        token: &str,
        id: &str,
        reason: &str,
        request: &str,
    ) -> Result<(String, Vec<u8>)> {
        text(&json!({"reason":reason}), "reason")?;
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        if state.user(&actor)?.status != "active" {
            return Err(Failure("FORBIDDEN", "账号不可查看敏感资料"));
        }
        if state.task_attachment_exists(id) {
            drop(tx);
            return crate::files::south::port::ReadEvidenceFile::new(self.repo.clone())
                .execute(token, id, reason, request)
                .await;
        }
        let mut owner = None;
        for a in state.settlement_applications().values() {
            let receipt = a["receipt_snapshot"]["file_id"] == id;
            let voucher = a["registration"]["voucher_file_id"] == id;
            if !receipt && !voucher {
                continue;
            }
            let target = text(a, "owner")?;
            if actor != target
                && state
                    .authorize(&actor, "settlement_reviewer", target)
                    .is_err()
                && state.authorize(&actor, "verifier", target).is_err()
                && !(a["handler"] == actor
                    && state.authorize(&actor, "manual_handler", target).is_ok())
            {
                continue;
            }
            owner = Some(if receipt {
                target.to_owned()
            } else {
                text(&a["registration"], "voucher_owner")?.to_owned()
            });
            break;
        }
        let result = match owner.as_deref() {
            Some(owner) => Repository::file(&mut tx, id, owner).await,
            None => Err(Failure("FORBIDDEN", "无本单收款资料或凭证查看权限")),
        };
        let now = Repository::time(&mut tx).await?;
        match &result {
            Ok(_) => state.log(&actor, "settlement_file_view", id, request, &now),
            Err(failure) => {
                state.log_failure(&actor, "settlement_file_view", id, request, &now, failure.0)
            }
        }
        {
            let mut log = serde_json::json!({});
            log["reason"] = json!(reason);

            state.annotate_audit(log);
        }
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        result
    }
}
