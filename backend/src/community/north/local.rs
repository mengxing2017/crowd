use crate::community::{
    domain::{Failure, Result},
    south::{adapter::postgres::Repository, port::IdentityProvider},
};
use serde_json::{Value, json};
#[derive(Clone)]
pub struct Service {
    pub repo: Repository,
}
impl Service {
    pub async fn staff_login(&self, input: &Value, request: &str) -> Result<Value> {
        let username = crate::community::domain::text(input, "username")?;
        let password = crate::community::domain::text(input, "password")?;
        self.repo.staff_login(username, password, request).await
    }

    pub async fn login(
        &self,
        provider: &impl IdentityProvider,
        input: &Value,
        request: &str,
    ) -> Result<Value> {
        let code = crate::community::domain::text(input, "code")?;
        let platform = provider.exchange(code).await.map_err(|_| {
            Failure(
                "IDENTITY_UNAVAILABLE",
                "微信凭证校验失败或身份服务暂不可用，请重新登录",
            )
        })?;
        self.repo.login(&platform, input, request).await
    }
    pub async fn agreements(&self) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let state = Repository::load(&mut tx).await?;
        let result = state
            .rules
            .iter()
            .find(|r| r["kind"] == "agreements" && r["status"] == "published")
            .cloned()
            .ok_or(Failure(
                "AGREEMENT_UNAVAILABLE",
                "服务协议与隐私说明尚未发布",
            ))?;
        Ok(result)
    }
    pub async fn resolve(&self, code: &str) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        Repository::load(&mut tx).await?.resolve(code)
    }
    pub async fn read(
        &self,
        token: &str,
        section: &str,
        target: Option<&str>,
        request: &str,
    ) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        let now = Repository::time(&mut tx).await?;
        let epoch = Repository::epoch(&mut tx).await?;
        let result = if crate::tasks::domain::is_read(section) {
            let result = state.tasks_read(&actor, section, target.unwrap_or(&actor), epoch);
            if result.is_ok() {
                state.log(&actor, section, target.unwrap_or(&actor), request, &now);
            }
            result
        } else if crate::catalog::domain::is_read(section) {
            let result = state.catalog_read(&actor, section, target.unwrap_or(&actor), epoch);
            if result.is_ok() {
                state.log(&actor, section, target.unwrap_or(&actor), request, &now);
            }
            result
        } else {
            state.read(&actor, section, target.unwrap_or(&actor), request, &now)
        };
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
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        result
    }
    pub async fn command(
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
            if action == "evidence_reveal" {
                let now = Repository::time(&mut tx).await?;
                state.log(
                    &actor,
                    "evidence_reveal",
                    input["id"].as_str().unwrap_or(""),
                    request,
                    &now,
                );
                if let Some(log) = state.audit.last_mut() {
                    log["reason"] = input["reason"].clone();
                }
                Repository::save(&mut tx, &state).await?;
                Repository::commit(tx).await?;
            }
            return Ok(result);
        }
        if action == "profile"
            && let Some(id) = input["avatar"].as_str()
        {
            Repository::validate_file(&mut tx, id, &actor, "avatar").await?;
        }
        if action == "receipt" && input["method"] == "wechat" {
            Repository::validate_file(
                &mut tx,
                crate::community::domain::text(input, "file_id")?,
                &actor,
                "wechat_receipt",
            )
            .await?;
        }
        Self::validate_catalog_files(&mut tx, &state, &actor, action, input).await?;
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
        let result = async {
            Ok::<Value, Failure>(if action == "logout" {
                Repository::logout(&mut tx, token).await?;
                json!({"status":"logged_out"})
            } else if action == "web_ticket" {
                Repository::web_ticket(&mut tx, &actor).await?
            } else if action == "receipt_reveal" {
                crate::community::domain::text(input, "reason")?;
                let id = crate::community::domain::text(input, "id")?;
                let receipt = state
                    .user(&actor)?
                    .receipts
                    .iter()
                    .find(|v| v["id"] == id)
                    .cloned()
                    .ok_or(Failure("NOT_FOUND", "资料不存在"))?;
                state.log(&actor, "REQ-03-008", id, request, &now);
                receipt
            } else if crate::tasks::domain::is_command(action) {
                let result = state.tasks_command(&actor, action, input, &generated, epoch)?;
                if action == "evidence_discard" {
                    let id = crate::community::domain::text(input, "id")?;
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
                                        submission["evidence"]["fields"].as_object().is_some_and(
                                            |fields| {
                                                fields.values().any(|value| {
                                                    value.as_array().is_some_and(|files| {
                                                        files.iter().any(|v| v == file.as_str())
                                                    })
                                                })
                                            },
                                        )
                                    })
                                })
                        })
                        .map(|(file, _)| file.clone())
                        .collect();
                    for file in unused {
                        Repository::delete_unused_evidence(&mut tx, &file, &actor).await?;
                        state.tasks.attachments.remove(&file);
                    }
                }
                state.log(
                    &actor,
                    action,
                    input["id"].as_str().unwrap_or(&actor),
                    request,
                    &now,
                );
                if let Some(audit) = state.audit.last_mut() {
                    audit["rule_version"] = json!("M08-M12-v1");
                    audit["reason"] = input["reason"].clone();
                    audit["object_version"] = input["version"].clone();
                    audit["result_ref"] = result["id"].clone();
                }
                result
            } else if crate::catalog::domain::is_command(action) {
                let result = state.catalog_command(&actor, action, input, &generated, epoch)?;
                state.log(
                    &actor,
                    action,
                    input["id"]
                        .as_str()
                        .or_else(|| input["user_id"].as_str())
                        .or_else(|| input["claim_id"].as_str())
                        .or_else(|| result["id"].as_str())
                        .unwrap_or(&actor),
                    request,
                    &now,
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
                        "product_review" =>
                            vec!["REQ-06-009", "REQ-07-003", "REQ-07-004", "REQ-07-007"],
                        "product_offline" => vec!["REQ-07-005", "REQ-07-006", "REQ-07-010"],
                        "product_archive" => vec!["REQ-06-010"],
                        "product_rollback" => vec!["REQ-07-008"],
                        "policy_publish" => vec!["REQ-05-002", "REQ-05-003"],
                        "touch_bind" => vec!["REQ-05-004"],
                        "task_claim" =>
                            vec!["REQ-05-005", "REQ-05-006", "REQ-05-007", "REQ-06-007"],
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
                result
            } else {
                state.command(&actor, action, input, &generated, request, &now)?
            })
        }
        .await;
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
    pub async fn upload(
        &self,
        token: &str,
        purpose: &str,
        bytes: Vec<u8>,
        request: &str,
    ) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        if state.user(&actor)?.status != "active" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号当前不可修改资料"));
        }
        if purpose == "product_image" {
            state.authorize(&actor, "operator", "platform")?;
        }
        let task_id = purpose.strip_prefix("task_evidence:");
        let size = bytes.len();
        if let Some(id) = task_id {
            let t = state
                .tasks
                .instances
                .get(id)
                .ok_or(Failure("NOT_FOUND", "任务不存在"))?;
            if t["owner"] != actor
                || !matches!(t["status"].as_str(), Some("in_progress" | "supplement"))
            {
                return Err(Failure("FORBIDDEN", "不能向该任务上传证据"));
            }
            if size > t["template"]["max_file_bytes"].as_u64().unwrap_or(0) as usize {
                return Err(Failure("INVALID_INPUT", "证据图片超过模板大小限制"));
            }
            let count = state
                .tasks
                .attachments
                .values()
                .filter(|a| a["task_id"] == id)
                .count();
            if count >= 100 {
                return Err(Failure(
                    "INVALID_INPUT",
                    "当前任务上传文件已达服务资源上限，请联系客服",
                ));
            }
        } else if purpose == "task_evidence" {
            return Err(Failure("INVALID_INPUT", "证据上传必须绑定任务"));
        }
        let result = Repository::upload(
            &mut tx,
            &actor,
            if task_id.is_some() {
                "task_evidence"
            } else {
                purpose
            },
            bytes,
        )
        .await?;
        if let Some(id) = task_id {
            state.tasks.attachments.insert(
                result["id"].as_str().unwrap().into(),
                json!({"task_id":id,"owner":actor,"size":size}),
            );
        }
        let now = Repository::time(&mut tx).await?;
        state.log(
            &actor,
            "file_upload",
            result["id"].as_str().unwrap(),
            request,
            &now,
        );
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(result)
    }
    pub async fn file(&self, token: &str, id: &str, request: &str) -> Result<(String, Vec<u8>)> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        if state.user(&actor)?.status != "active" {
            return Err(Failure("FORBIDDEN", "账号不可查看私有资料"));
        }
        let result = Repository::file(&mut tx, id, &actor).await?;
        let now = Repository::time(&mut tx).await?;
        state.log(&actor, "file_view", id, request, &now);
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(result)
    }
    pub async fn evidence_file(
        &self,
        token: &str,
        id: &str,
        reason: &str,
        request: &str,
    ) -> Result<(String, Vec<u8>)> {
        let mut tx = self.repo.begin().await?;
        let actor = Repository::authenticate(&mut tx, token).await?;
        let mut state = Repository::load(&mut tx).await?;
        crate::community::domain::text(&json!({"reason":reason}), "reason")?;
        let a = state
            .tasks
            .attachments
            .get(id)
            .ok_or(Failure("NOT_FOUND", "证据图片不存在"))?;
        let t = state
            .tasks
            .instances
            .get(a["task_id"].as_str().unwrap_or(""))
            .ok_or(Failure("NOT_FOUND", "任务不存在"))?;
        if state.user(&actor)?.status != "active" {
            return Err(Failure("FORBIDDEN", "账号不可查看证据"));
        }
        if a["owner"] != actor {
            let target = t["product_id"].as_str().unwrap_or("");
            let reviewer = state.authorize(&actor, "task_reviewer", target).is_ok()
                && t["reviewer"] == actor
                && matches!(
                    t["status"].as_str(),
                    Some("reviewing" | "awaiting_verification")
                );
            if !reviewer && state.authorize(&actor, "verifier", target).is_err() {
                return Err(Failure("FORBIDDEN", "无本单证据查看权限"));
            }
            let referenced = t["submissions"].as_array().is_some_and(|versions| {
                versions.iter().any(|v| {
                    v["evidence"]["fields"].as_object().is_some_and(|fields| {
                        fields.values().any(|value| {
                            value
                                .as_array()
                                .is_some_and(|files| files.iter().any(|f| f == id))
                        })
                    })
                })
            });
            if !referenced {
                return Err(Failure("FORBIDDEN", "不能查看未提交的草稿附件"));
            }
        }
        let owner = a["owner"].as_str().unwrap_or("").to_owned();
        let result = Repository::file(&mut tx, id, &owner).await?;
        let now = Repository::time(&mut tx).await?;
        state.log(&actor, "evidence_file_view", id, request, &now);
        if let Some(log) = state.audit.last_mut() {
            log["reason"] = json!(reason);
        }
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(result)
    }
}
