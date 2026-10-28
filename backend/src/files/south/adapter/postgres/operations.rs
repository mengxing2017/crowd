use crate::platform::south::adapter::postgres::{Repository, database};
use crate::shared::{Failure, Result, invalid};
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};
impl Repository {
    pub async fn validate_file(
        tx: &mut Transaction<'_, Postgres>,
        id: &str,
        actor: &str,
        purpose: &str,
    ) -> Result<()> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM crowd_files WHERE id=$1 AND owner_id=$2 AND purpose=$3)",
        )
        .bind(id)
        .bind(actor)
        .bind(purpose)
        .fetch_one(&mut **tx)
        .await
        .map_err(database)?;
        if exists {
            Ok(())
        } else {
            Err(Failure("FORBIDDEN", "文件不存在、用途错误或不属于本人"))
        }
    }
    pub async fn upload(
        tx: &mut Transaction<'_, Postgres>,
        actor: &str,
        purpose: &str,
        bytes: Vec<u8>,
    ) -> Result<Value> {
        if !["avatar", "wechat_receipt", "product_image", "task_evidence"].contains(&purpose) {
            return Err(invalid("文件用途无效"));
        }
        // Engineering resource ceiling, not a confirmed business image policy.
        if bytes.len() > 5 * 1024 * 1024 {
            return Err(invalid("图片超过当前服务处理上限"));
        }
        let format =
            image::guess_format(&bytes).map_err(|_| invalid("仅支持真实 PNG/JPEG 图片"))?;
        let mime = match format {
            image::ImageFormat::Png => "image/png",
            image::ImageFormat::Jpeg => "image/jpeg",
            _ => return Err(invalid("仅支持 PNG/JPEG 图片")),
        };
        let mut reader = image::ImageReader::with_format(std::io::Cursor::new(&bytes), format);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(4096);
        limits.max_image_height = Some(4096);
        limits.max_alloc = Some(64 * 1024 * 1024);
        reader.limits(limits);
        reader
            .decode()
            .map_err(|_| invalid("图片损坏或解码资源超限"))?;
        let id:String=sqlx::query_scalar("INSERT INTO crowd_files(owner_id,purpose,mime,content) VALUES($1,$2,$3,$4) RETURNING id").bind(actor).bind(purpose).bind(mime).bind(bytes).fetch_one(&mut **tx).await.map_err(database)?;
        Ok(json!({"id":id,"purpose":purpose,"official_verified":false}))
    }
    pub async fn file(
        tx: &mut Transaction<'_, Postgres>,
        id: &str,
        actor: &str,
    ) -> Result<(String, Vec<u8>)> {
        sqlx::query_as("SELECT mime,content FROM crowd_files WHERE id=$1 AND owner_id=$2")
            .bind(id)
            .bind(actor)
            .fetch_optional(&mut **tx)
            .await
            .map_err(database)?
            .ok_or(Failure("FORBIDDEN", "无权查看文件"))
    }
}
