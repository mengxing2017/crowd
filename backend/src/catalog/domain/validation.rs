use crate::community::domain::{Failure, Result, State, invalid, text};
use serde_json::{Value, json};

pub(super) fn num(v: &Value, field: &str) -> Result<i64> {
    v[field]
        .as_i64()
        .filter(|v| v.unsigned_abs() <= 9_007_199_254_740_991)
        .ok_or(invalid("整数或时间字段缺失，金额必须使用安全整数分"))
}
pub(super) fn strings(v: &Value) -> Result<Vec<String>> {
    serde_json::from_value(v.clone()).map_err(|_| invalid("需提供字符串数组"))
}
pub(super) fn version(v: &Value) -> u64 {
    v["version"].as_u64().unwrap_or(0)
}
pub(super) fn not_found() -> Failure {
    Failure("NOT_FOUND", "业务对象不存在")
}
pub(super) fn conflict() -> Failure {
    Failure("VERSION_CONFLICT", "状态或版本已改变，请刷新")
}
pub(super) fn public_category(category: &Value) -> Value {
    json!({"id":category["id"],"version":category["version"],"name":category["name"],"sort":category["sort"],"enabled":category["enabled"]})
}

impl State {
    pub(super) fn validate_content(&self, c: &Value, now: i64, live: bool) -> Result<()> {
        for field in ["name", "category", "summary", "details", "limitations"] {
            text(c, field)?;
        }
        let cat = self
            .catalog
            .categories
            .get(text(c, "category")?)
            .ok_or(invalid("分类不存在"))?;
        if cat["enabled"] != true {
            return Err(invalid("分类已停用"));
        }
        strings(&c["regions"])?;
        strings(&c["qualifications"])?;
        if num(c, "estimated_minutes")? <= 0 {
            return Err(invalid("预计时长必须为正整数"));
        }
        let images = c["images"].as_array().ok_or(invalid("需提供图片数组"))?;
        if images.is_empty() || images.len() > 20 {
            return Err(invalid("需提供 1–20 张产品图片（当前工程上限）"));
        }
        for image in images {
            text(image, "file_id")?;
            text(image, "alt")?;
        }
        let tasks = c["tasks"]
            .as_array()
            .filter(|t| !t.is_empty())
            .ok_or(invalid("至少关联一个任务模板"))?;
        let mut ids = std::collections::BTreeSet::new();
        for t in tasks {
            if !ids.insert(text(t, "id")?) {
                return Err(invalid("任务模板编号重复"));
            }
            for f in ["name", "completion", "evidence", "rule_version"] {
                text(t, f)?;
            }
            if num(t, "version")? <= 0
                || num(t, "commission_cents")? < 0
                || t["currency"] != "CNY"
                || num(t, "submit_seconds")? <= 0
            {
                return Err(invalid("任务版本、整数佣金、币种或提交时限无效"));
            }
            if num(t, "rule_starts_at")? >= num(t, "rule_ends_at")?
                || (live && (now < num(t, "rule_starts_at")? || now >= num(t, "rule_ends_at")?))
            {
                return Err(invalid("佣金规则未生效或已过期"));
            }
            if !t["allow_self"].is_boolean() {
                return Err(invalid("必须明确是否允许自推"));
            }
            for f in ["capacity", "digital_limit_cents"] {
                if !t[f].is_null() && num(t, f)? < 0 {
                    return Err(invalid("名额或计划数字上限无效"));
                }
            }
        }
        Ok(())
    }
    pub(super) fn task_available(&self, product: &str, task: &Value, now: i64) -> bool {
        let count = self
            .catalog
            .claims
            .values()
            .filter(|c| {
                c["product_id"] == product
                    && c["task"]["id"] == task["id"]
                    && c["reservation_released"] != true
            })
            .count() as i64;
        let used = self
            .catalog
            .claims
            .values()
            .filter(|c| {
                c["product_id"] == product
                    && c["task"]["id"] == task["id"]
                    && c["reservation_released"] != true
            })
            .try_fold(0i64, |acc, c| {
                acc.checked_add(c["task"]["commission_cents"].as_i64().unwrap_or(0))
            });
        task["template_id"].as_str().is_none_or(|id| {
            self.tasks.templates.get(id).is_some_and(|v| {
                v["status"] == "approved"
                    && v["content"]["claim_until"]
                        .as_i64()
                        .is_some_and(|end| now < end)
            })
        }) && task["rule_starts_at"].as_i64().is_some_and(|t| t <= now)
            && task["rule_ends_at"].as_i64().is_some_and(|t| now < t)
            && task["capacity"].as_i64().is_none_or(|n| count < n)
            && task["digital_limit_cents"].as_i64().is_none_or(|n| {
                used.and_then(|u| u.checked_add(task["commission_cents"].as_i64().unwrap_or(0)))
                    .is_some_and(|u| u <= n)
            })
    }
}
