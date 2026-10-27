use crate::{
    catalog::domain,
    community::{
        domain::{Failure, Result, text},
        north::local::Service,
        south::adapter::postgres::Repository,
    },
};
use serde_json::{Value, json};
impl Service {
    pub async fn reconcile_catalog(&self) -> Result<()> {
        let mut tx = self.repo.begin().await?;
        let mut state = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        let revision = state.catalog.revision;
        state.reconcile_lifecycle(now);
        if revision != state.catalog.revision {
            let time = Repository::time(&mut tx).await?;
            state.log(
                "catalog-worker",
                "REQ-07-004+REQ-07-009+REQ-07-010",
                "platform",
                "lifecycle-worker",
                &time,
            );
            Repository::save(&mut tx, &state).await?;
        }
        Repository::commit(tx).await
    }
    pub async fn public_catalog(&self, query: &Value) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let state = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        state.public_catalog(query, now)
    }
    pub async fn touch(&self, input: &Value, key: &str, request: &str) -> Result<Value> {
        let mut tx = self.repo.begin().await?;
        let mut state = Repository::load(&mut tx).await?;
        let idempotency = format!("guest-touch:{key}");
        if let Some(previous) = state.idempotency.get(&idempotency) {
            if previous["input"] != *input {
                return Err(Failure("IDEMPOTENCY_CONFLICT", "同一访问键内容不同"));
            }
            return Ok(previous["result"].clone());
        }
        let generated = Repository::entropy(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        let value = state.collect_touch(input, &generated, now)?;
        state
            .idempotency
            .insert(idempotency, json!({"input":input,"result":value}));
        let time = Repository::time(&mut tx).await?;
        state.log("guest", "REQ-05-001", "visit", request, &time);
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(value)
    }
    pub async fn validate_catalog_files(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        state: &crate::community::domain::State,
        actor: &str,
        action: &str,
        input: &Value,
    ) -> Result<()> {
        let content = if action == "product_save" {
            Some(&input["content"])
        } else {
            None
        };
        if let Some(content) = content {
            for image in content["images"].as_array().into_iter().flatten() {
                let id = text(image, "file_id")?;
                // Reusing an existing product image is allowed only within the already-authorized product.
                let prior = state
                    .catalog
                    .products
                    .get(input["id"].as_str().unwrap_or(""));
                let linked = prior.is_some_and(|p| {
                    p["draft"]["images"]
                        .as_array()
                        .is_some_and(|images| images.iter().any(|v| v["file_id"] == id))
                        || p["releases"].as_array().is_some_and(|rs| {
                            rs.iter().any(|r| {
                                r["content"]["images"]
                                    .as_array()
                                    .is_some_and(|images| images.iter().any(|v| v["file_id"] == id))
                            })
                        })
                });
                if !linked {
                    Repository::validate_file(tx, id, actor, "product_image").await?;
                }
            }
        }
        Ok(())
    }
    pub async fn product_image(&self, id: &str) -> Result<(String, Vec<u8>)> {
        let mut tx = self.repo.begin().await?;
        let state = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        let visible = state.catalog.products.keys().any(|p| {
            state.visible_release(p, now).is_ok_and(|r| {
                r["content"]["images"]
                    .as_array()
                    .is_some_and(|images| images.iter().any(|v| v["file_id"] == id))
            })
        });
        if !visible {
            return Err(Failure("NOT_FOUND", "公开产品图片不存在"));
        }
        Repository::public_product_image(&mut tx, id).await
    }
}
// Keep classification centralized for north adapters without duplicating domain rules.
pub fn is_command(action: &str) -> bool {
    domain::is_command(action)
}

impl Service {
    /// Internal port only. The caller must supply a verified task source, never raw HTTP data.
    pub async fn record_verified_completion(
        &self,
        source: &str,
        event: &str,
        claim: &str,
        occurred_at: i64,
        request: &str,
    ) -> Result<Value> {
        use crate::catalog::south::port::CompletionRecorder;
        let mut tx = self.repo.begin().await?;
        let mut state = Repository::load(&mut tx).await?;
        let now = Repository::epoch(&mut tx).await?;
        if occurred_at > now {
            return Err(Failure("INVALID_INPUT", "完成事件时间不能晚于服务端时间"));
        }
        let value = state.record_completion(source, event, claim, occurred_at)?;
        let time = Repository::time(&mut tx).await?;
        state.log(source, "REQ-05-010", claim, request, &time);
        Repository::save(&mut tx, &state).await?;
        Repository::commit(tx).await?;
        Ok(value)
    }
}
