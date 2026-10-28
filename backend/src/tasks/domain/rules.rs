use super::validation::{
    active, array, conflict, missing, next_time, number, positive, require, version,
};
use crate::platform::domain::{Failure, Result, State, invalid, text};
use serde_json::{Value, json};

/// All arithmetic is integral; proportional inputs come exclusively from a verified source.
pub fn calculate(rule: &Value, base: Option<i64>) -> Result<Value> {
    let raw = match rule["kind"].as_str() {
        Some("fixed") => {
            let n = number(rule, "amount_cents")?;
            require(n >= 0, "固定佣金不得为负数")?;
            i128::from(n)
        }
        Some("proportional") => {
            let base = base.ok_or(Failure(
                "VERIFIED_BASE_REQUIRED",
                "比例佣金须由可信成果提供计算基数",
            ))?;
            require(
                (0..=9_007_199_254_740_991).contains(&base),
                "计算基数须为非负安全整数",
            )?;
            let numerator = number(rule, "numerator")?;
            let denominator = positive(rule, "denominator")?;
            require(
                numerator >= 0 && numerator <= denominator,
                "比例须介于零和一之间",
            )?;
            let product = i128::from(base) * i128::from(numerator);
            let denominator = i128::from(denominator);
            match rule["rounding"].as_str() {
                Some("floor") => product / denominator,
                Some("ceil") => (product + denominator - 1) / denominator,
                Some("half_up") => (product + denominator / 2) / denominator,
                _ => return Err(invalid("须明确比例舍入方式")),
            }
        }
        _ => return Err(invalid("定价方式错误")),
    };
    let amount = if rule["cap_cents"].is_null() {
        raw
    } else {
        let cap = number(rule, "cap_cents")?;
        require(cap >= 0, "封顶数字不得为负数")?;
        raw.min(i128::from(cap))
    };
    require(amount <= 9_007_199_254_740_991, "计算结果超出安全整数范围")?;
    Ok(
        json!({"base_cents":base,"numerator":rule["numerator"],"denominator":rule["denominator"],"rounding":rule["rounding"],"raw_cents":raw.to_string(),"cap_cents":rule["cap_cents"],"amount_cents":amount as i64,"currency":"CNY","rule_id":rule["id"],"rule_version":rule["version"]}),
    )
}
fn validate_price(p: &Value) -> Result<()> {
    text(p, "task_id")?;
    text(p, "settlement_scope")?;
    if !p["group"].is_null() {
        text(p, "group")?;
    }
    require(p["currency"] == "CNY", "当前仅支持人民币整数分")?;
    require(
        number(p, "starts_at")? < number(p, "ends_at")?,
        "规则起止时间错误",
    )?;
    match p["kind"].as_str() {
        Some("fixed") => {
            calculate(p, None)?;
        }
        Some("proportional") => {
            text(p, "base_source")?;
            calculate(p, Some(1))?;
        }
        _ => return Err(invalid("请选择固定或比例佣金")),
    }
    if !p["period_cap_cents"].is_null() {
        require(number(p, "period_cap_cents")? >= 0, "周期上限不得为负数")?;
        positive(p, "period_seconds")?;
        number(p, "period_anchor")?;
        require(
            p["reservation_policy"] == "reserve_max_release_terminal",
            "须明确占用与终止释放策略",
        )?;
        if p["kind"] == "proportional" {
            require(
                !p["cap_cents"].is_null(),
                "设置周期上限的比例规则必须明确单笔封顶",
            )?;
        }
    }
    Ok(())
}
fn validate_template(t: &Value) -> Result<()> {
    for f in [
        "name",
        "task_type",
        "completion",
        "failure_conditions",
        "review_goal",
    ] {
        text(t, f)?;
    }
    for f in [
        "execution_seconds",
        "submit_seconds",
        "total_quota",
        "per_user_quota",
        "review_sla_seconds",
        "max_evidence_files",
        "max_file_bytes",
    ] {
        positive(t, f)?;
    }
    require(
        positive(t, "max_evidence_files")? <= 20
            && positive(t, "max_file_bytes")? <= 5 * 1024 * 1024,
        "文件限制超过平台上限",
    )?;
    number(t, "claim_until")?;
    require(
        number(t, "execution_seconds")? <= number(t, "submit_seconds")?,
        "执行时限不能晚于提交时限",
    )?;
    require(
        matches!(t["source"].as_str(), Some("manual" | "external" | "both")),
        "成果来源错误",
    )?;
    require(
        matches!(
            t["participation"].as_str(),
            Some("once" | "period" | "repeat")
        ),
        "参与频次错误",
    )?;
    if t["participation"] == "period" {
        positive(t, "period_seconds")?;
        number(t, "period_anchor")?;
    }
    for f in [
        "allow_abandon",
        "release_on_terminal",
        "allow_withdraw",
        "double_review",
    ] {
        require(t[f].is_boolean(), "须明确放弃、释放、撤回和复核策略")?;
    }
    require(
        matches!(t["pause_policy"].as_str(), Some("extend" | "keep_deadline")),
        "须明确暂停的截止时间策略",
    )?;
    require(number(t, "max_supplements")? >= 0, "补交次数错误")?;
    positive(t, "supplement_seconds")?;
    require(!array(t, "steps")?.is_empty(), "任务至少包含一个步骤")?;
    let mut ids = std::collections::BTreeSet::new();
    for step in array(t, "steps")? {
        require(ids.insert(text(step, "id")?), "步骤编号重复")?;
        text(step, "title")?;
        require(step["required"].is_boolean(), "须明确步骤是否必填")?;
    }
    ids.clear();
    for field in array(t, "fields")? {
        require(ids.insert(text(field, "id")?), "证据字段编号重复")?;
        text(field, "label")?;
        text(field, "purpose")?;
        require(field["required"].is_boolean(), "须明确证据字段是否必填")?;
        require(
            matches!(field["kind"].as_str(), Some("text" | "image" | "order")),
            "不支持此证据类型",
        )?;
        if field["kind"] == "text" {
            positive(field, "max_length")?;
        }
        if field["kind"] == "order" {
            text(field, "source")?;
            text(field, "exclusive_scope")?;
        }
    }
    for field in ["regions", "qualifications"] {
        require(
            array(t, field)?.iter().all(|v| {
                v.as_str()
                    .is_some_and(|s| !s.trim().is_empty() && s.len() <= 128)
            }),
            "地区与资格必须为非空标签数组",
        )?;
    }
    if t["source"] != "manual" {
        text(t, "external_source")?;
    }
    Ok(())
}
impl State {
    pub(super) fn rule_command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        now: i64,
    ) -> Result<Value> {
        let price = action.starts_with("price_");
        let id = input["id"].as_str().unwrap_or(generated);
        let map = if price {
            &self.tasks.prices
        } else {
            &self.tasks.templates
        };
        if action.ends_with("_save") {
            let previous = map.get(id);
            require(
                input["version"].as_u64().unwrap_or(0) == previous.map_or(0, version),
                "规则版本已改变",
            )?;
            let content = input["content"].clone();
            if price {
                validate_price(&content)?;
            } else {
                validate_template(&content)?;
            }
            // A saved version is immutable, including after rejection. Updates create another object.
            require(previous.is_none(), "版本不可覆盖，请用新编号创建新版本")?;
            let value = json!({"id":id,"version":1,"content":content,"editor":actor,"created_at":now,"status":"pending","history":[]});
            if price {
                self.tasks.prices.insert(id.into(), value.clone());
            } else {
                self.tasks.templates.insert(id.into(), value.clone());
            }
            return Ok(value);
        }
        let mut old = map.get(id).cloned().ok_or(missing())?;
        if version(&old) != input["version"].as_u64().unwrap_or(0) {
            return Err(conflict());
        }
        text(input, "reason")?;
        if action == "template_disable" {
            require(old["status"] == "approved", "仅可停用已批准模板")?;
            old["status"] = json!("disabled");
        } else {
            require(
                old["status"] == "pending" && old["editor"] != actor,
                "必须由另一人员独立审核待审版本",
            )?;
            require(input["approve"].is_boolean(), "须明确审核结论")?;
            if input["approve"] == true && price {
                let c = &old["content"];
                for other in self
                    .tasks
                    .prices
                    .values()
                    .filter(|p| p["status"] == "approved")
                {
                    let o = &other["content"];
                    if c["task_id"] == o["task_id"]
                        && c["settlement_scope"] == o["settlement_scope"]
                        && c["group"] == o["group"]
                        && number(c, "starts_at")? < number(o, "ends_at")?
                        && number(o, "starts_at")? < number(c, "ends_at")?
                    {
                        return Err(Failure(
                            "RULE_OVERLAP",
                            "同任务、范围和人群的生效时间与已批准规则重叠",
                        ));
                    }
                }
            }
            old["status"] = json!(if input["approve"] == true {
                "approved"
            } else {
                "rejected"
            });
        }
        let entry = json!({"actor":actor,"action":action,"at":now,"reason":input["reason"],"status":old["status"],"version":version(&old)+1});
        old["history"].as_array_mut().unwrap().push(entry);
        if action == "template_disable" {
            old["disabled_by"] = json!(actor);
            old["disabled_reason"] = input["reason"].clone();
            old["disabled_at"] = json!(now);
        } else {
            old["reviewer"] = json!(actor);
            old["reason"] = input["reason"].clone();
            old["reviewed_at"] = json!(now);
        }
        old["version"] = json!(version(&old) + 1);
        if price {
            self.tasks.prices.insert(id.into(), old.clone());
        } else {
            self.tasks.templates.insert(id.into(), old.clone());
        }
        Ok(old)
    }
    pub fn task_snapshot(&self, actor: &str, task: &Value, now: i64) -> Result<Option<Value>> {
        let Some(template_id) = task["template_id"].as_str() else {
            return Ok(None);
        };
        let template = self
            .tasks
            .templates
            .get(template_id)
            .filter(|v| v["status"] == "approved")
            .ok_or(Failure("DEPENDENCY_UNAVAILABLE", "任务模板未批准或已停用"))?;
        let price = self
            .tasks
            .prices
            .get(text(task, "price_id")?)
            .filter(|v| v["status"] == "approved")
            .ok_or(Failure("DEPENDENCY_UNAVAILABLE", "佣金规则尚未批准"))?;
        let t = &template["content"];
        let p = &price["content"];
        require(p["task_id"] == task["id"], "佣金规则不适用于此任务")?;
        require(
            number(p, "starts_at")? <= now
                && now < number(p, "ends_at")?
                && now < number(t, "claim_until")?,
            "规则或领取时间已到期",
        )?;
        let qualification = self
            .catalog
            .eligibility
            .get(actor)
            .cloned()
            .unwrap_or(Value::Null);
        require(
            array(t, "regions")?.is_empty()
                || array(t, "regions")?.contains(&qualification["region"]),
            "地区不满足核验条件",
        )?;
        require(
            array(t, "qualifications")?.iter().all(|q| {
                qualification["qualifications"]
                    .as_array()
                    .is_some_and(|v| v.contains(q))
            }),
            "资格不满足核验条件",
        )?;
        if let Some(group) = p["group"].as_str() {
            require(
                qualification["qualifications"]
                    .as_array()
                    .is_some_and(|v| v.iter().any(|q| q == group)),
                "不适用此人群定价",
            )?;
        }
        let reserve = if p["kind"] == "fixed" {
            number(&calculate(p, None)?, "amount_cents")?
        } else {
            p["cap_cents"].as_i64().unwrap_or(0)
        };
        require(
            task["commission_cents"] == reserve && task["currency"] == "CNY",
            "产品展示数字与批准规则不一致",
        )?;
        let prior: Vec<_> = self
            .tasks
            .instances
            .values()
            .filter(|v| {
                v["template_id"] == template_id
                    && (active(&v["status"]) || v["reservation_released"] != true)
            })
            .collect();
        require(
            (prior.len() as i64) < number(t, "total_quota")?,
            "任务模板总名额不足",
        )?;
        let own: Vec<_> = self
            .catalog
            .claims
            .values()
            .filter(|v| v["owner"] == actor && v["task"]["id"] == task["id"])
            .collect();
        let period_start = if t["participation"] == "period" {
            let d = positive(t, "period_seconds")?;
            let a = number(t, "period_anchor")?;
            a + (now - a).div_euclid(d) * d
        } else {
            i64::MIN
        };
        let counted = own
            .iter()
            .filter(|v| {
                v["locked_at"].as_i64().unwrap_or(0) >= period_start
                    && (v["reservation_released"] != true || t["participation"] == "once")
            })
            .count() as i64;
        let limit = if t["participation"] == "once" {
            1
        } else {
            number(t, "per_user_quota")?
        };
        require(counted < limit, "已达到本任务参与次数或个人名额限制")?;
        if !p["period_cap_cents"].is_null() {
            let d = positive(p, "period_seconds")?;
            let a = number(p, "period_anchor")?;
            let start = a + (now - a).div_euclid(d) * d;
            let used: i128 = self
                .tasks
                .instances
                .values()
                .filter(|v| {
                    v["owner"] == actor
                        && v["price"]["settlement_scope"] == p["settlement_scope"]
                        && v["locked_at"].as_i64().unwrap_or(0) >= start
                        && v["reservation_released"] != true
                })
                .map(|v| i128::from(v["reserved_cents"].as_i64().unwrap_or(0)))
                .sum();
            require(
                used + i128::from(reserve) <= i128::from(number(p, "period_cap_cents")?),
                "用户周期佣金计划上限不足",
            )?;
        }
        let mut frozen_price = p.clone();
        frozen_price["id"] = price["id"].clone();
        frozen_price["version"] = price["version"].clone();
        Ok(Some(
            json!({"template_id":template_id,"template_version":template["version"],"template":t,"price":frozen_price,"qualification":qualification,"reserved_cents":reserve}),
        ))
    }
    pub fn attach_task(&mut self, claim: &Value, snapshot: Option<Value>, now: i64) -> Result<()> {
        let Some(snapshot) = snapshot else {
            return Ok(());
        };
        let mut t = claim.clone();
        for (k, v) in snapshot.as_object().unwrap() {
            t[k] = v.clone();
        }
        t["version"] = json!(1);
        t["steps"] = json!([]);
        t["submissions"] = json!([]);
        t["decisions"] = json!([]);
        t["events"] = json!([]);
        t["supplement_count"] = json!(0);
        t["reservation_released"] = json!(false);
        t["execute_until"] = json!(next_time(
            now,
            positive(&t["template"], "execution_seconds")?
        )?);
        t["submit_until"] = json!(next_time(now, positive(&t["template"], "submit_seconds")?)?);
        self.tasks.instances.insert(text(claim, "id")?.into(), t);
        Ok(())
    }
}

impl State {
    pub fn validate_task_reference(&self, task: &mut Value, start: i64, end: i64) -> Result<()> {
        let Some(id) = task["template_id"].as_str() else {
            return Ok(());
        };
        let template = self
            .tasks
            .templates
            .get(id)
            .filter(|v| v["status"] == "approved")
            .ok_or(Failure("DEPENDENCY_UNAVAILABLE", "任务模板未批准或已停用"))?;
        let price = self
            .tasks
            .prices
            .get(text(task, "price_id")?)
            .filter(|v| v["status"] == "approved")
            .ok_or(Failure("DEPENDENCY_UNAVAILABLE", "定价规则未批准"))?;
        let p = &price["content"];
        require(
            p["task_id"] == task["id"]
                && number(p, "starts_at")? <= start
                && number(p, "ends_at")? >= end,
            "定价规则不适用此任务或不能覆盖发布区间",
        )?;
        let displayed = if p["kind"] == "fixed" {
            number(&calculate(p, None)?, "amount_cents")?
        } else {
            p["cap_cents"].as_i64().unwrap_or(0)
        };
        require(
            task["completion"] == template["content"]["completion"]
                && task["commission_cents"] == displayed
                && task["submit_seconds"] == template["content"]["submit_seconds"],
            "产品展示佣金或提交时限与批准版本不一致",
        )?;
        require(
            p["kind"] != "proportional"
                || !p["cap_cents"].is_null()
                || task["digital_limit_cents"].is_null(),
            "未封顶比例无法预占活动数字上限，请明确封顶或不配置该上限",
        )?;
        task["template_preview"] = template["content"].clone();
        task["price_preview"] = p.clone();
        Ok(())
    }
    pub fn expire_tasks(&mut self, now: i64) -> usize {
        let reminders: Vec<_> = self
            .tasks
            .instances
            .iter()
            .filter(|(_, t)| {
                matches!(
                    t["status"].as_str(),
                    Some("submitted" | "reviewing" | "awaiting_verification")
                ) && now - t["submitted_at"].as_i64().unwrap_or(now)
                    > t["template"]["review_sla_seconds"]
                        .as_i64()
                        .unwrap_or(i64::MAX)
            })
            .map(|(id, t)| {
                (
                    id.clone(),
                    t["receipt"]["id"].clone(),
                    t["product_id"].as_str().unwrap_or("").to_owned(),
                    t["reviewer"].as_str().map(String::from),
                    t["status"].clone(),
                    t["owner"].clone(),
                    t["first_reviewer"].clone(),
                )
            })
            .collect();
        for (id, receipt, product, assigned, phase, owner, first) in reminders {
            let key = json!([id, receipt, phase]).to_string();
            if self.tasks.reminders.contains(&key) {
                continue;
            }
            let recipients: Vec<_> = self
                .users
                .keys()
                .filter(|actor| {
                    if phase == "awaiting_verification" {
                        owner != actor.as_str()
                            && first != actor.as_str()
                            && self.authorize(actor, "verifier", &product).is_ok()
                    } else {
                        assigned.as_ref().is_none_or(|a| a == *actor)
                            && self.authorize(actor, "task_reviewer", &product).is_ok()
                    }
                })
                .cloned()
                .collect();
            for recipient in recipients {
                self.catalog.revision += 1;
                self.catalog.notifications.push(json!({"recipient":recipient,"product_id":product,"task_id":id,"kind":"review_overdue","time":now,"revision":self.catalog.revision}));
            }
            self.tasks.reminders.push(key);
        }

        let ids: Vec<_> = self
            .tasks
            .instances
            .iter()
            .filter(|(_, t)| {
                matches!(
                    t["status"].as_str(),
                    Some("claimed" | "in_progress" | "supplement")
                ) && now
                    > if t["status"] == "supplement" {
                        t["supplement_until"].as_i64().unwrap_or(i64::MAX)
                    } else {
                        t["submit_until"].as_i64().unwrap_or(i64::MAX)
                    }
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in &ids {
            let t = self.tasks.instances.get_mut(id).unwrap();
            t["status"] = json!("expired");
            t["reservation_released"] = json!(t["template"]["release_on_terminal"] == true);
            t["version"] = json!(version(t) + 1);
            t["events"]
                .as_array_mut()
                .unwrap()
                .push(json!({"action":"timeout","at":now,"actor":"task-worker"}));
            if let Some(c) = self.catalog.claims.get_mut(id) {
                c["status"] = json!("expired");
                c["reservation_released"] = t["reservation_released"].clone();
            }
        }
        ids.len()
    }
}
