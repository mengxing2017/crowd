use crate::platform::domain::{Failure, Result};
use crate::platform::south::adapter::postgres::Repository;
use serde_json::{Value, json};

#[derive(Clone)]
pub struct ExecuteCommand {
    repo: Repository,
}

impl ExecuteCommand {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        token: &str,
        action: &str,
        input: &Value,
        key: &str,
        request: &str,
    ) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        if let Err(failure) = state.check_command_access(&actor, action, input) {
            let now = Repository::time(&mut tx).await?;
            state.log_failure(
                &actor,
                action,
                input["id"].as_str().unwrap_or(&actor),
                request,
                &now,
                failure.0,
            );
            Repository::save(&mut tx, &state).await?;
            Repository::commit(tx).await?;
            return Err(failure);
        }
        let idempotency = format!("{actor}:{key}");
        if let Some(previous) = state.idempotency.get(&idempotency) {
            if previous["action"] != action || previous["input"] != *input {
                return Err(Failure("IDEMPOTENCY_CONFLICT", "同一请求键内容不同"));
            }
            let result = previous["result"].clone();
            if crate::tasks::north::local::audit_replay(
                &mut tx, &mut state, &actor, action, input, request,
            )
            .await?
            {
                Repository::save(&mut tx, &state).await?;
                Repository::commit(tx).await?;
            }
            return Ok(result);
        }
        super::input_files::validate(&mut tx, &state, &actor, action, input).await?;
        let epoch = Repository::epoch(&mut tx).await?;
        let expired = state.expire_tasks(epoch);
        let generated = Repository::entropy(&mut tx).await?;
        let now = Repository::time(&mut tx).await?;
        if expired > 0 {
            state.log("task-worker", "task_expire", "platform", request, &now);
            if let Some(audit) = state.audit.last_mut() {
                audit["rule_version"] = json!("M08-M12-v1");
                audit["expired_tasks"] = json!(expired);
            }
        }
        let before = state.clone();
        let context = super::command_context::CommandContext {
            actor: &actor,
            action,
            input,
            generated: &generated,
            request,
            now: &now,
            epoch,
        };
        let result = super::command_dispatch::apply(&mut tx, &mut state, token, &context).await;
        let result = match result {
            Ok(value) => value,
            Err(failure) => {
                state = before;
                state.log_failure(&actor, action, &actor, request, &now, failure.0);
                Repository::save(&mut tx, &state).await?;
                Repository::commit(tx).await?;
                return Err(failure);
            }
        };
        if action == "logout" || action == "web_ticket" {
            state.log(&actor, action, &actor, request, &now);
        }
        state.idempotency.insert(
            idempotency,
            json!({"action":action,"input":input,"result":result}),
        );
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(result)
    }
}
