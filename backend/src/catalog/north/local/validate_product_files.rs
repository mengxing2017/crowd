use crate::catalog::south::port::DomainPort;
use crate::catalog::south::port::Repository;
use crate::shared::Result;
use crate::shared::text;
use serde_json::Value;

pub struct ValidateProductFiles;

impl ValidateProductFiles {
    pub async fn execute<P: DomainPort + ?Sized>(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        state: &P,
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
                    .catalog()
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
}
