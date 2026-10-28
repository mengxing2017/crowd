use crate::catalog::south::port::AuditWriter;
use crate::catalog::south::port::Repository;
use crate::shared::Result;
use serde_json::json;

#[derive(Clone)]
pub struct ReconcileCatalog {
    repo: Repository,
}

impl ReconcileCatalog {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<()> {
        let mut tx = self.repo.begin().await?;
        let mut state = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        let revision = state.catalog.revision;
        state.reconcile_lifecycle(now);
        let expired = state.expire_tasks(now);
        if revision != state.catalog.revision || expired > 0 {
            let time = Repository::time(&mut tx).await?;
            state.log(
                "catalog-worker",
                "REQ-07-004+REQ-07-009+REQ-07-010+REQ-10-007+REQ-12-007",
                "platform",
                "lifecycle-worker",
                &time,
            );
            {
                let mut audit = serde_json::json!({});
                audit["rule_version"] = json!("M05-M12-v1");
                audit["expired_tasks"] = json!(expired);

                state.annotate_audit(audit);
            }
            Repository::save(&mut tx, &state).await?;
        }
        Repository::commit(tx).await
    }
}
