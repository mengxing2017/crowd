use super::validation::{conflict, not_found, num, public_category, strings, version};
use crate::community::domain::{Failure, Result, State, check_version, invalid, text};
use serde_json::{Value, json};

impl State {
    pub fn catalog_command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        now: i64,
    ) -> Result<Value> {
        self.catalog_access(actor, action, input)?;
        let id = input["id"].as_str().unwrap_or(generated).to_owned();
        let result = match action {
            "category_save" => {
                text(input, "name")?;
                num(input, "sort")?;
                text(input, "reason")?;
                if !input["enabled"].is_boolean() {
                    return Err(invalid("需明确分类启停状态"));
                }
                let old = self
                    .catalog
                    .categories
                    .get(&id)
                    .cloned()
                    .unwrap_or(json!({"version":0}));
                check_version(version(&old), input)?;
                let mut history = old["history"].as_array().cloned().unwrap_or_default();
                if version(&old) > 0 {
                    let mut previous = old.clone();
                    previous.as_object_mut().unwrap().remove("history");
                    history.push(previous);
                }
                let mut v = input.clone();
                v["history"] = json!(history);
                v["id"] = json!(id);
                v["version"] = json!(version(&old) + 1);
                self.catalog.categories.insert(id.clone(), v);
                self.catalog.revision += 1;
                json!({"id":id,"version":version(&old)+1})
            }
            "category_migrate" => {
                let to = text(input, "to_category")?;
                text(input, "reason")?;
                if self
                    .catalog
                    .categories
                    .get(to)
                    .is_none_or(|c| c["enabled"] != true)
                {
                    return Err(invalid("目标分类不存在或已停用"));
                }
                let category = self.catalog.categories.get(&id).ok_or(not_found())?;
                check_version(version(category), input)?;
                // Only drafts move. Published versions and historical claims are immutable.
                for p in self.catalog.products.values_mut() {
                    if p["draft"]["category"] == id {
                        p["draft"]["category"] = json!(to);
                        p["version"] = json!(version(p) + 1);
                        p["editor"] = json!(actor);
                    }
                }
                self.catalog.categories.get_mut(&id).unwrap()["version"] =
                    json!(version(category) + 1);
                json!({"status":"drafts_migrated","notice":"已发布版本需重新提交审核"})
            }
            "product_save" => {
                if !input["content"].is_object() {
                    return Err(invalid("产品草稿必须为对象"));
                }
                let p=self.catalog.products.entry(id.clone()).or_insert(json!({"id":id,"version":0,"draft":{},"releases":[],"archived":false,"offline":true}));
                check_version(version(p), input)?;
                if p["archived"] == true {
                    return Err(conflict());
                }
                p["draft"] = input["content"].clone();
                p["version"] = json!(version(p) + 1);
                p["editor"] = json!(actor);
                json!({"id":id,"version":version(p)})
            }
            "product_rollback" => {
                text(input, "reason")?;
                let p = self.catalog.products.get(&id).ok_or(not_found())?;
                check_version(version(p), input)?;
                if p["archived"] == true {
                    return Err(conflict());
                }
                let release = p["releases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| {
                        r["id"] == input["release_id"]
                            && (r["status"] == "approved" || r["status"] == "withdrawn")
                    })
                    .cloned()
                    .ok_or(not_found())?;
                let p = self.catalog.products.get_mut(&id).unwrap();
                p["draft"] = release["content"].clone();
                p["editor"] = json!(actor);
                p["version"] = json!(version(p) + 1);
                json!({"id":id,"version":version(p),"status":"draft","notice":"回滚创建草稿，须重新校验并独立审核发布"})
            }
            "product_submit" => {
                let mut p = self.catalog.products.get(&id).cloned().ok_or(not_found())?;
                check_version(version(&p), input)?;
                if p["archived"] == true
                    || p["releases"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|r| r["status"] == "pending")
                {
                    return Err(conflict());
                }
                self.validate_content(&p["draft"], now, false)?;
                let start = num(input, "starts_at")?;
                let end = num(input, "ends_at")?;
                if start >= end || end <= now {
                    return Err(invalid("发布有效期无效"));
                }
                for task in p["draft"]["tasks"].as_array_mut().unwrap() {
                    self.validate_task_reference(task, start, end)?;
                    if num(task, "rule_starts_at")? > start || num(task, "rule_ends_at")? < end {
                        return Err(invalid("佣金规则有效期必须覆盖发布有效期"));
                    }
                }
                let r = json!({"id":generated,"content":p["draft"],"starts_at":start,"ends_at":end,"status":"pending","submitted_by":actor,"editor":p["editor"],"submitted_at":now});
                let p = self.catalog.products.get_mut(&id).unwrap();
                p["releases"].as_array_mut().unwrap().push(r);
                p["version"] = json!(version(p) + 1);
                json!({"id":id,"release_id":generated,"version":version(p),"status":"pending"})
            }
            "product_review" => {
                text(input, "reason")?;
                let p = self.catalog.products.get(&id).cloned().ok_or(not_found())?;
                check_version(version(&p), input)?;
                let pos = p["releases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .position(|r| r["id"] == input["release_id"] && r["status"] == "pending")
                    .ok_or(conflict())?;
                let r = &p["releases"][pos];
                if r["editor"] == actor || r["submitted_by"] == actor {
                    return Err(Failure(
                        "FORBIDDEN",
                        "产品编辑、提交与发布审核须由不同人员执行",
                    ));
                }
                if input["approved"] == true {
                    self.validate_content(&r["content"], now, false)?;
                    if num(r, "ends_at")? <= now || p["archived"] == true {
                        return Err(invalid("待审版本已到期或产品已归档"));
                    }
                    for other in p["releases"].as_array().unwrap() {
                        if other["status"] == "approved"
                            && num(other, "starts_at")? < num(r, "ends_at")?
                            && num(r, "starts_at")? < num(other, "ends_at")?
                            && other["starts_at"].as_i64().is_some_and(|t| t > now)
                        {
                            return Err(invalid("与已批准的定时发布有效期冲突"));
                        }
                    }
                } else if input["approved"] != false {
                    return Err(invalid("需明确审核结论"));
                }
                let p = self.catalog.products.get_mut(&id).unwrap();
                p["releases"][pos]["status"] = json!(if input["approved"] == true {
                    "approved"
                } else {
                    "rejected"
                });
                p["releases"][pos]["reviewer"] = json!(actor);
                p["releases"][pos]["review_reason"] = input["reason"].clone();
                if input["approved"] == true {
                    p["offline"] = json!(false);
                }
                p["version"] = json!(version(p) + 1);
                self.notification(&id, "release_reviewed", now);
                json!({"id":id,"version":version(&self.catalog.products[&id])})
            }
            "product_offline" | "product_archive" => {
                text(input, "reason")?;
                if action == "product_offline" && input["stock_policy"] != "continue_existing" {
                    return Err(Failure(
                        "DEPENDENCY_UNAVAILABLE",
                        "当前仅支持停止新领取并保留存量；终止任务需后续任务模块接入",
                    ));
                }
                if input["emergency"] == true {
                    text(input, "risk_id")?;
                    text(input, "impact")?;
                }
                let p = self.catalog.products.get_mut(&id).ok_or(not_found())?;
                check_version(version(p), input)?;
                p["offline"] = json!(true);
                p["version"] = json!(version(p) + 1);
                if action == "product_archive" {
                    p["archived"] = json!(true);
                }
                p["offline_reason"] = input["reason"].clone();
                p["stock_policy"] = json!("continue_existing");
                // Approved schedules are cancelled, never silently resurrected after an emergency.
                for r in p["releases"].as_array_mut().unwrap() {
                    if r["status"] == "approved" {
                        r["status"] = json!("withdrawn");
                    }
                }
                self.notification(&id, action, now);
                json!({"id":id,"version":version(&self.catalog.products[&id]),"stock_policy":"continue_existing"})
            }
            "policy_publish" => {
                let window = num(input, "window_seconds")?;
                if window <= 0 || !["first_touch", "last_touch"].contains(&text(input, "strategy")?)
                {
                    return Err(invalid("明确配置正整数归因窗口及首触或末触策略"));
                }
                check_version(self.catalog.policies.len() as u64, input)?;
                let p = json!({"id":generated,"version":self.catalog.policies.len()+1,"window_seconds":window,"strategy":input["strategy"],"starts_at":now,"owner":actor});
                self.catalog.policies.push(p.clone());
                p
            }
            "touch_bind" => self.bind_touch(actor, text(input, "touch_token")?, now)?,
            "eligibility_verify" => {
                let target = text(input, "user_id")?;
                self.user(target)?;
                if target == actor {
                    return Err(Failure("FORBIDDEN", "不能核验自己的地区或资格"));
                }
                text(input, "reason")?;
                let old = self
                    .catalog
                    .eligibility
                    .get(target)
                    .cloned()
                    .unwrap_or(json!({"version":0,"history":[]}));
                check_version(version(&old), input)?;
                let region = text(input, "region")?;
                strings(&input["qualifications"])?;
                let mut history = old["history"].as_array().cloned().unwrap_or_default();
                history.push(json!({"region":region,"qualifications":input["qualifications"],"verified_by":actor,"reason":input["reason"],"time":now}));
                self.catalog.eligibility.insert(target.into(),json!({"version":version(&old)+1,"region":region,"qualifications":input["qualifications"],"history":history}));
                json!({"version":version(&old)+1})
            }
            "task_claim" => {
                if input["confirmed"] != true {
                    return Err(invalid("须主动确认任务条件和数字佣金"));
                }
                if input
                    .as_object()
                    .unwrap()
                    .keys()
                    .any(|k| !["id", "task_id", "release_id", "confirmed"].contains(&k.as_str()))
                {
                    return Err(invalid("领取不能指定推广人、地区、佣金或身份"));
                }
                let r = self.visible_release(&id, now)?;
                if r["id"] != input["release_id"] {
                    return Err(conflict());
                }
                let task = r["content"]["tasks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|t| t["id"] == input["task_id"])
                    .cloned()
                    .ok_or(not_found())?;
                let execution_snapshot = self.task_snapshot(actor, &task, now)?;
                if execution_snapshot.is_none()
                    && self.catalog.claims.values().any(|c| {
                        c["owner"] == actor
                            && c["release_id"] == r["id"]
                            && c["task"]["id"] == task["id"]
                    })
                {
                    return Err(Failure(
                        "ALREADY_CLAIMED",
                        "本发布版本的任务已领取，请查询原记录",
                    ));
                }
                if !self.task_available(&id, &task, now) {
                    return Err(Failure(
                        "QUOTA_EXHAUSTED",
                        "名额或计划数字上限不足，或规则已到期",
                    ));
                }
                let regions = strings(&r["content"]["regions"])?;
                let required = strings(&r["content"]["qualifications"])?;
                let eligibility = self
                    .catalog
                    .eligibility
                    .get(actor)
                    .cloned()
                    .unwrap_or(Value::Null);
                if (!regions.is_empty() && !regions.iter().any(|s| eligibility["region"] == *s))
                    || required.iter().any(|q| {
                        !eligibility["qualifications"]
                            .as_array()
                            .is_some_and(|v| v.iter().any(|v| v == q))
                    })
                {
                    return Err(Failure("INELIGIBLE", "未满足经核验的地区或资格条件"));
                }
                let policy = self
                    .catalog
                    .policies
                    .last()
                    .cloned()
                    .ok_or(Failure("DEPENDENCY_UNAVAILABLE", "归因策略尚未配置"))?;
                let touch = self.select_touch(actor, &id, text(&task, "id")?, now, &policy);
                if touch.as_ref().is_some_and(|t| t["promoter"] == actor)
                    && task["allow_self"] != true
                {
                    return Err(Failure("SELF_PROMOTION_FORBIDDEN", "此任务不允许本人自推"));
                }
                let category = self
                    .catalog
                    .categories
                    .get(text(&r["content"], "category")?)
                    .cloned()
                    .unwrap();
                let claim = json!({"id":generated,"owner":actor,"product_id":id,"release_id":r["id"],"product":r["content"],"category":public_category(&category),"task":task,"policy":policy,"touch":touch,"locked_at":now,"submit_until":now.checked_add(num(&task,"submit_seconds")?).ok_or(invalid("时限超出范围"))?,"status":"claimed","notice":"领取仅固定条件和归因，不生成可申请佣金"});
                self.attach_task(&claim, execution_snapshot, now)?;
                self.catalog.claims.insert(generated.into(), claim.clone());
                claim
            }
            "attribution_dispute" => {
                text(input, "reason")?;
                text(input, "evidence")?;
                let c = self
                    .catalog
                    .claims
                    .get(text(input, "claim_id")?)
                    .ok_or(not_found())?;
                if c["owner"] != actor && c["touch"]["promoter"] != actor {
                    return Err(Failure("FORBIDDEN", "无权申诉此任务"));
                }
                let d = json!({"id":generated,"owner":actor,"claim_id":input["claim_id"],"reason":input["reason"],"evidence":input["evidence"],"original_touch":c["touch"],"status":"pending","version":1,"time":now});
                self.catalog.disputes.insert(generated.into(), d.clone());
                d
            }
            "dispute_resolve" => {
                text(input, "reason")?;
                let decision = text(input, "decision")?;
                if !["uphold", "needs_adjustment"].contains(&decision) {
                    return Err(invalid("裁定只能维持归属或转关联数字调整核查"));
                }
                let d = self.catalog.disputes.get_mut(&id).ok_or(not_found())?;
                check_version(version(d), input)?;
                if d["owner"] == actor {
                    return Err(Failure("FORBIDDEN", "不能裁定自己的申诉"));
                }
                if d["status"] != "pending" {
                    return Err(conflict());
                }
                d["status"] = json!(decision);
                d["reviewer"] = json!(actor);
                d["decision_reason"] = input["reason"].clone();
                d["version"] = json!(version(d) + 1);
                d.clone()
            }
            _ => return Err(not_found()),
        };
        Ok(result)
    }
}
