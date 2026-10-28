use crate::platform::domain::{Failure, Result};
use crate::platform::south::adapter::postgres::Repository;

#[derive(Clone)]
pub struct ReadProductImage {
    repo: Repository,
}

impl ReadProductImage {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &str) -> Result<(String, Vec<u8>)> {
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
