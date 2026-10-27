use super::*;
fn editable(t: &Value, now: i64) -> Result<()> {
    require(
        matches!(t["status"].as_str(), Some("in_progress" | "supplement")),
        "当前状态不能编辑或提交证据",
    )?;
    require(now <= number(t, "submit_until")?, "证据提交期限已过")?;
    if t["status"] == "supplement" {
        require(now <= number(t, "supplement_until")?, "补交期限已过")?;
    }
    Ok(())
}
fn order_key(field: &Value, value: &str) -> String {
    json!([
        field["exclusive_scope"],
        field["source"],
        value.trim().to_ascii_uppercase()
    ])
    .to_string()
}
impl State {
    fn validate_evidence(&self, t: &Value, e: &Value, complete: bool) -> Result<()> {
        let object = e["fields"]
            .as_object()
            .ok_or(invalid("证据 fields 必须为对象"))?;
        let schema = array(&t["template"], "fields")?;
        require(
            object
                .keys()
                .all(|key| schema.iter().any(|f| f["id"] == key.as_str())),
            "证据包含模板未定义字段",
        )?;
        let mut file_count = 0;
        for field in schema {
            let id = text(field, "id")?;
            let value = &e["fields"][id];
            if value.is_null() {
                require(
                    !complete || field["required"] != true,
                    "必填证据缺失，请按字段标签补齐",
                )?;
                continue;
            }
            match field["kind"].as_str() {
                Some("image") => {
                    let values = value
                        .as_array()
                        .ok_or(invalid("图片字段必须是文件编号数组"))?;
                    file_count += values.len();
                    require(
                        !complete || field["required"] != true || !values.is_empty(),
                        "必填图片字段不可为空",
                    )?;
                    for file in values {
                        let attachment = self
                            .tasks
                            .attachments
                            .get(file.as_str().ok_or(invalid("图片文件编号无效"))?)
                            .ok_or(Failure("FORBIDDEN", "证据图片未授权"))?;
                        require(
                            attachment["owner"] == t["owner"] && attachment["task_id"] == t["id"],
                            "证据图片必须属于本人和当前任务",
                        )?;
                    }
                }
                Some("text") => {
                    let text = value.as_str().ok_or(invalid("文本证据格式错误"))?;
                    require(
                        text.chars().count() <= positive(field, "max_length")? as usize,
                        "文本证据超出字段长度限制",
                    )?;
                    require(
                        !complete || field["required"] != true || !text.trim().is_empty(),
                        "必填文本证据不可为空",
                    )?;
                }
                Some("order") => {
                    let value = value.as_str().ok_or(invalid("订单号必须是文本"))?;
                    require(
                        !value.trim().is_empty()
                            && value.len() <= 128
                            && value
                                .trim()
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c)),
                        "订单号仅支持字母、数字、横线和下划线，最长128字节",
                    )?;
                    let key = order_key(field, value);
                    require(
                        self.tasks
                            .orders
                            .get(&key)
                            .is_none_or(|id| t["id"] == id.as_str()),
                        "同来源订单已被互斥任务提交，请申请核查",
                    )?;
                }
                _ => return Err(invalid("未知证据字段类型")),
            }
        }
        require(
            file_count <= positive(&t["template"], "max_evidence_files")? as usize,
            "图片数量超过模板限制",
        )?;
        if complete {
            let occurred = number(e, "occurred_at")?;
            require(
                occurred >= number(t, "started_at")? && occurred <= number(t, "execute_until")?,
                "证据发生时间必须位于任务实际开始和执行截止之间",
            )?;
            if t["status"] == "supplement" {
                let previous = t["submissions"]
                    .as_array()
                    .and_then(|a| a.last())
                    .ok_or(missing())?;
                require(
                    e["occurred_at"] == previous["evidence"]["occurred_at"],
                    "补交不得改写原成果发生时间",
                )?;
                for field in array(&t["template"], "fields")? {
                    let key = text(field, "id")?;
                    if !array(t, "supplement_fields")?.iter().any(|f| f == key) {
                        require(
                            e["fields"][key] == previous["evidence"]["fields"][key],
                            "补交只能修改审核指定字段",
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
    pub(super) fn execution_command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        now: i64,
    ) -> Result<Value> {
        let id = text(input, "id")?;
        let mut t = self.tasks.instances.get(id).cloned().ok_or(missing())?;
        if input["version"].as_u64() != Some(version(&t)) {
            return Err(conflict());
        }
        match action {
            "task_start" => {
                require(
                    t["status"] == "claimed" && now <= number(&t, "execute_until")?,
                    "任务已开始或已超过执行期限",
                )?;
                t["status"] = json!("in_progress");
                if t["started_at"].is_null() {
                    t["started_at"] = json!(now);
                }
            }
            "task_step" => {
                editable(&t, now)?;
                require(now <= number(&t, "execute_until")?, "执行期限已过")?;
                let steps = array(&t["template"], "steps")?;
                let step = text(input, "step_id")?;
                let pos = steps
                    .iter()
                    .position(|s| s["id"] == step)
                    .ok_or(invalid("步骤不属于当前模板"))?;
                require(
                    steps[..pos]
                        .iter()
                        .filter(|s| s["required"] == true)
                        .all(|s| array(&t, "steps").is_ok_and(|a| a.contains(&s["id"]))),
                    "请先完成之前的必做步骤",
                )?;
                if !array(&t, "steps")?.iter().any(|s| s == step) {
                    t["steps"].as_array_mut().unwrap().push(json!(step));
                }
            }
            "task_abandon" => {
                require(
                    matches!(t["status"].as_str(), Some("claimed" | "in_progress"))
                        && t["template"]["allow_abandon"] == true,
                    "此阶段或模板不允许主动放弃",
                )?;
                text(input, "reason")?;
                t["status"] = json!("abandoned");
                t["reservation_released"] = json!(t["template"]["release_on_terminal"] == true);
            }
            "task_pause" => {
                require(
                    matches!(
                        t["status"].as_str(),
                        Some("claimed" | "in_progress" | "supplement")
                    ),
                    "当前阶段不可暂停",
                )?;
                text(input, "reason")?;
                t["resume_status"] = t["status"].clone();
                t["status"] = json!("paused");
                t["paused_at"] = json!(now);
            }
            "task_resume" => {
                require(t["status"] == "paused", "任务未暂停")?;
                text(input, "reason")?;
                let duration = now - number(&t, "paused_at")?;
                require(duration >= 0, "暂停时长错误")?;
                if t["template"]["pause_policy"] == "extend" {
                    for field in ["execute_until", "submit_until", "supplement_until"] {
                        if let Some(n) = t[field].as_i64() {
                            t[field] = json!(next_time(n, duration)?);
                        }
                    }
                }
                t["status"] = t["resume_status"].clone();
            }
            "task_expire" => {
                require(
                    matches!(
                        t["status"].as_str(),
                        Some("claimed" | "in_progress" | "supplement")
                    ),
                    "审核中、暂停或已结束任务不得自动失效",
                )?;
                let deadline = if t["status"] == "supplement" {
                    number(&t, "supplement_until")?
                } else {
                    number(&t, "submit_until")?
                };
                require(now > deadline, "任务尚未超时")?;
                t["status"] = json!("expired");
                t["reservation_released"] = json!(t["template"]["release_on_terminal"] == true);
            }
            "evidence_discard" => {
                editable(&t, now)?;
                t["draft"] = Value::Null;
                t["preview"] = Value::Null;
            }
            "evidence_save" => {
                editable(&t, now)?;
                self.validate_evidence(&t, &input["evidence"], false)?;
                t["draft"] = input["evidence"].clone();
                let order_fields: Vec<String> = array(&t["template"], "fields")?
                    .iter()
                    .filter(|f| f["kind"] == "order")
                    .filter_map(|f| f["id"].as_str().map(String::from))
                    .collect();
                for field in order_fields {
                    if let Some(value) = t["draft"]["fields"][&field].as_str() {
                        let normalized = value.trim().to_ascii_uppercase();
                        t["draft"]["fields"][&field] = json!(normalized);
                    }
                }

                t["draft_saved_at"] = json!(now);
                t["preview"] = Value::Null;
            }
            "evidence_preview" => {
                editable(&t, now)?;
                self.validate_evidence(&t, &t["draft"], true)?;
                require(
                    number(&t["draft"], "occurred_at")? <= now,
                    "成果发生时间不能晚于服务器当前时间",
                )?;
                t["preview"] = json!({"id":generated,"evidence":t["draft"],"at":now});
            }
            "evidence_submit" => {
                editable(&t, now)?;
                require(input["confirmed"] == true, "须确认最终证据和真实性声明")?;
                require(
                    !t["preview"].is_null() && t["preview"]["id"] == input["preview_id"],
                    "请先预览当前证据版本",
                )?;
                self.validate_evidence(&t, &t["draft"], true)?;
                require(
                    array(&t["template"], "steps")?
                        .iter()
                        .filter(|s| s["required"] == true)
                        .all(|s| array(&t, "steps").is_ok_and(|a| a.contains(&s["id"]))),
                    "仍有必做步骤未完成",
                )?;
                for field in array(&t["template"], "fields")?
                    .iter()
                    .filter(|f| f["kind"] == "order")
                {
                    if let Some(value) = t["draft"]["fields"][text(field, "id")?].as_str() {
                        self.tasks.orders.insert(order_key(field, value), id.into());
                    }
                }
                let n = array(&t, "submissions")?.len() + 1;
                let submission = json!({"id":generated,"version":n,"evidence":t["draft"],"submitted_at":now,"declaration_confirmed":true});
                t["submissions"]
                    .as_array_mut()
                    .unwrap()
                    .push(submission.clone());
                t["receipt"] = submission;
                t["submitted_at"] = json!(now);
                t["submission_origin"] = t["status"].clone();
                t["status"] = json!("submitted");
                t["reviewer"] = Value::Null;
                t["preview"] = Value::Null;
            }
            "evidence_withdraw" => {
                require(
                    t["status"] == "submitted" && t["template"]["allow_withdraw"] == true,
                    "已被审核领取或模板不允许撤回",
                )?;
                require(now <= number(&t, "submit_until")?, "提交期限已过，不能撤回")?;
                text(input, "reason")?;
                if t["submission_origin"] == "supplement" {
                    require(
                        now <= number(&t, "supplement_until")?,
                        "补交期限已过，不能撤回",
                    )?;
                    t["status"] = json!("supplement");
                } else {
                    t["status"] = json!("in_progress");
                }
                t["preview"] = Value::Null;
            }
            _ => return Err(invalid("不支持的任务动作")),
        }
        t["version"] = json!(version(&t) + 1);
        let v = t["version"].clone();
        t["events"].as_array_mut().unwrap().push(
            json!({"action":action,"actor":actor,"at":now,"reason":input["reason"],"version":v}),
        );
        self.tasks.instances.insert(id.into(), t.clone());
        if let Some(c) = self.catalog.claims.get_mut(id) {
            c["status"] = t["status"].clone();
            c["reservation_released"] = t["reservation_released"].clone();
        }
        Ok(t)
    }
}
