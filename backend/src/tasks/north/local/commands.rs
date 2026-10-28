use crate::platform::south::adapter::postgres::Repository;
use crate::platform::{
    domain::{Failure, Result, State},
    north::local::CommandContext,
};
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};

pub(crate) async fn apply(
    tx: &mut Transaction<'_, Postgres>,
    state: &mut State,
    context: &CommandContext<'_>,
) -> Result<Value> {
    let CommandContext {
        actor,
        action,
        input,
        generated,
        request,
        now,
        epoch,
    } = *context;
    let result = state.tasks_command(actor, action, input, generated, epoch)?;
    if action == "evidence_discard" {
        let id = crate::platform::domain::text(input, "id")?;
        let task = state
            .tasks
            .instances
            .get(id)
            .ok_or(Failure("NOT_FOUND", "任务不存在"))?;
        let unused: Vec<String> = state
            .tasks
            .attachments
            .iter()
            .filter(|(file, a)| {
                a["task_id"] == id
                    && a["owner"] == actor
                    && !task["submissions"].as_array().is_some_and(|submissions| {
                        submissions.iter().any(|submission| {
                            submission["evidence"]["fields"]
                                .as_object()
                                .is_some_and(|fields| {
                                    fields.values().any(|value| {
                                        value.as_array().is_some_and(|files| {
                                            files.iter().any(|v| v == file.as_str())
                                        })
                                    })
                                })
                        })
                    })
            })
            .map(|(file, _)| file.clone())
            .collect();
        for file in unused {
            Repository::delete_unused_evidence(tx, &file, actor).await?;
            state.tasks.attachments.remove(&file);
        }
    }
    state.log(
        actor,
        action,
        input["id"].as_str().unwrap_or(actor),
        request,
        now,
    );
    if let Some(audit) = state.audit.last_mut() {
        audit["rule_version"] = json!("M08-M12-v1");
        audit["reason"] = input["reason"].clone();
        audit["object_version"] = input["version"].clone();
        audit["result_ref"] = result["id"].clone();
    }
    Ok(result)
}
