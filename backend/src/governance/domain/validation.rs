use crate::shared::{Result, invalid, text};
use serde_json::Value;
pub(super) fn validate_catalogue(content: &Value) -> Result<()> {
    let products = content["products"]
        .as_array()
        .ok_or(invalid("产品任务目录需提供 products 数组"))?;
    let mut product_ids = std::collections::BTreeSet::new();
    let mut task_ids = std::collections::BTreeSet::new();
    for p in products {
        if !product_ids.insert(text(p, "id")?) {
            return Err(invalid("产品编号重复"));
        }
        if !["active", "inactive"].contains(&text(p, "status")?) {
            return Err(invalid("产品状态无效"));
        }
        for t in p["tasks"].as_array().ok_or(invalid("需提供 tasks 数组"))? {
            if !task_ids.insert(text(t, "id")?) {
                return Err(invalid("任务编号重复"));
            }
            if !["active", "inactive"].contains(&text(t, "status")?) {
                return Err(invalid("任务状态无效"));
            }
            text(t, "completion")?;
            text(t, "evidence")?;
        }
    }
    Ok(())
}
