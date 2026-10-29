use crate::platform::south::port::Repository;
use crate::shared::Result;
use serde_json::Value;

#[derive(Clone)]
pub struct ReadResource {
    repo: Repository,
}

impl ReadResource {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        token: &str,
        section: &str,
        target: Option<&str>,
        request: &str,
    ) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        state.user(&actor)?;
        let now = Repository::time(&mut tx).await?;
        let epoch = Repository::epoch(&mut tx).await?;
        let audit_count = state.audit.len();
        let result = crate::platform::south::port::query_dispatch::read(
            &mut state, &actor, section, target, request, &now, epoch,
        );
        if let Err(ref failure) = result {
            state.log_failure(
                &actor,
                section,
                target.unwrap_or(&actor),
                request,
                &now,
                failure.0,
            );
        }
        if result.is_ok() && state.audit.len() == audit_count {
            state.log(&actor, section, target.unwrap_or(&actor), request, &now);
        }
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        result
    }
}
