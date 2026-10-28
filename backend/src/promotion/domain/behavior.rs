use super::Code;
use crate::platform::domain::State;
use crate::shared::{Failure, Result, invalid};
use serde_json::{Value, json};
impl State {
    pub fn register(
        &mut self,
        id: &str,
        code: &str,
        agreement: Value,
        source: Option<String>,
    ) -> Result<()> {
        if self.users.contains_key(id) {
            return Ok(());
        }
        if self.codes.contains_key(code) {
            return Err(Failure("CODE_COLLISION", "推广码冲突，请重试"));
        }
        self.register_user(id, agreement, source);
        self.codes.insert(
            code.into(),
            Code {
                owner: id.into(),
                version: 1,
                active: true,
                product: None,
                task: None,
            },
        );
        Ok(())
    }
    pub fn resolve(&self, code: &str) -> Result<Value> {
        let c = self
            .codes
            .get(code)
            .filter(|c| c.active)
            .ok_or(Failure("CODE_UNAVAILABLE", "推广码不存在或已撤销"))?;
        let u = self.user(&c.owner)?;
        if u.status != "active" || u.restrictions.iter().any(|r| r["scope"] == "promotion") {
            return Err(Failure("CODE_UNAVAILABLE", "推广账号当前不可用"));
        }
        self.validate_target(c.product.as_deref(), c.task.as_deref())?;
        Ok(
            json!({"code":code,"version":c.version,"product":c.product,"task":c.task,"notice":"推广访问不产生佣金数字"}),
        )
    }
    pub fn validate_target(&self, product: Option<&str>, task: Option<&str>) -> Result<()> {
        let Some(product) = product else {
            return if task.is_none() {
                Ok(())
            } else {
                Err(invalid("任务入口必须指定产品"))
            };
        };
        if self.catalog.products.contains_key(product) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let release = self.visible_release(product, now)?;
            if task.is_some_and(|task| {
                !release["content"]["tasks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|t| t["id"] == task)
            }) {
                return Err(Failure("TARGET_UNAVAILABLE", "目标任务不存在"));
            }
            return Ok(());
        }
        let catalogue = self
            .rules
            .iter()
            .find(|r| r["kind"] == "product_tasks" && r["status"] == "published")
            .ok_or(Failure("DEPENDENCY_UNAVAILABLE", "产品任务目录尚未发布"))?;
        let p = catalogue["content"]["products"]
            .as_array()
            .and_then(|items| items.iter().find(|p| p["id"] == product))
            .ok_or(Failure("TARGET_UNAVAILABLE", "目标产品不存在"))?;
        if p["status"] != "active" {
            return Err(Failure("TARGET_UNAVAILABLE", "目标产品已下架或停用"));
        }
        if let Some(task) = task {
            let t = p["tasks"]
                .as_array()
                .and_then(|items| items.iter().find(|t| t["id"] == task))
                .ok_or(Failure("TARGET_UNAVAILABLE", "目标任务不存在"))?;
            if t["status"] != "active" {
                return Err(Failure("TARGET_UNAVAILABLE", "目标任务已停用"));
            }
        }
        Ok(())
    }
}
