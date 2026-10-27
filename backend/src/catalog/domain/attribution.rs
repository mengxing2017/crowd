use crate::community::domain::{Failure, Result, State, invalid, text};
use serde_json::{Value, json};

impl State {
    pub fn collect_touch(&mut self, input: &Value, generated: &str, now: i64) -> Result<Value> {
        // Deliberately no user/promoter input. Ownership is derived from an active server code.
        let object = input.as_object().ok_or(invalid("请求必须为对象"))?;
        if object
            .keys()
            .any(|k| !["code", "scene"].contains(&k.as_str()))
        {
            return Err(invalid("访问仅接受码值与场景"));
        }
        let code = text(input, "code")?;
        text(input, "scene")?;
        let c = self
            .codes
            .get(code)
            .cloned()
            .ok_or(Failure("CODE_UNAVAILABLE", "推广码无效，请返回产品列表"))?;
        if !c.active
            || self.users.get(&c.owner).is_none_or(|u| {
                u.status != "active" || u.restrictions.iter().any(|r| r["scope"] == "promotion")
            })
        {
            return Err(Failure("CODE_UNAVAILABLE", "推广码已失效，请返回产品列表"));
        }
        if let Some(p) = &c.product {
            let r = self.visible_release(p, now)?;
            if c.task.as_ref().is_some_and(|t| {
                !r["content"]["tasks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v["id"] == *t)
            }) {
                return Err(Failure("TARGET_UNAVAILABLE", "推广任务已停用"));
            }
        }
        self.catalog.touches.insert(generated.into(),json!({"id":generated,"code":code,"code_version":c.version,"promoter":c.owner,"product":c.product,"task":c.task,"scene":input["scene"],"time":now,"user":null}));
        Ok(
            json!({"touch_token":generated,"product":c.product,"task":c.task,"notice":"访问不占名额，也不产生佣金数字"}),
        )
    }
    pub fn bind_touch(&mut self, actor: &str, token: &str, now: i64) -> Result<Value> {
        let t = self
            .catalog
            .touches
            .get_mut(token)
            .ok_or(Failure("TOUCH_INVALID", "触点不存在"))?;
        if t["user"].is_string() && t["user"] != actor {
            return Err(Failure("FORBIDDEN", "触点已关联其他身份"));
        }
        // Unbound bearer token is kept only in memory by the client; never returned in query APIs.
        if now < t["time"].as_i64().unwrap_or(now) {
            return Err(invalid("触点时间无效"));
        }
        t["user"] = json!(actor);
        Ok(json!({"status":"bound"}))
    }
    pub(super) fn select_touch(
        &self,
        actor: &str,
        product: &str,
        task: &str,
        now: i64,
        policy: &Value,
    ) -> Option<Value> {
        let window = policy["window_seconds"].as_i64()?;
        let mut touches: Vec<_> = self
            .catalog
            .touches
            .values()
            .filter(|t| {
                t["user"] == actor
                    && (t["product"].is_null() || t["product"] == product)
                    && (t["task"].is_null() || t["task"] == task)
                    && t["time"]
                        .as_i64()
                        .is_some_and(|at| at <= now && now.saturating_sub(at) <= window)
            })
            .collect();
        touches.sort_by(|a, b| {
            (a["time"].as_i64(), a["id"].as_str()).cmp(&(b["time"].as_i64(), b["id"].as_str()))
        });
        if policy["strategy"] == "first_touch" {
            touches.first()
        } else {
            touches.last()
        }
        .map(|t| (*t).clone())
    }
}
