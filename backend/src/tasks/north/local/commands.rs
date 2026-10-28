use crate::shared::{Failure, Result};
use crate::tasks::domain::*;
use crate::tasks::south::port::CommandContext;
use crate::tasks::south::port::DomainPort;
use crate::tasks::south::port::Repository;
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};

pub(crate) async fn apply<P: DomainPort + ?Sized>(
    tx: &mut Transaction<'_, Postgres>,
    state: &mut P,
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
        let id = crate::shared::text(input, "id")?;
        let task = state
            .tasks()
            .instances
            .get(id)
            .ok_or(Failure("NOT_FOUND", "任务不存在"))?;
        let unused: Vec<String> = state
            .tasks()
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
            state.tasks_mut().attachments.remove(&file);
        }
    }
    state.log(
        actor,
        action,
        input["id"].as_str().unwrap_or(actor),
        request,
        now,
    );
    {
        let mut audit = serde_json::json!({});
        audit["rule_version"] = json!("M08-M12-v1");
        audit["reason"] = input["reason"].clone();
        audit["object_version"] = input["version"].clone();
        audit["result_ref"] = result["id"].clone();

        state.annotate_audit(audit);
    }
    Ok(result)
}
