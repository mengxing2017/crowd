use super::*;
impl State {
    fn approve_task(&mut self, t: &mut Value, actor: &str, now: i64) -> Result<()> {
        let id = text(t, "id")?.to_owned();
        require(
            !self.tasks.confirmations.contains_key(&id),
            "此实例已有有效佣金确认，不得重复确认",
        )?;
        let source = text(&t["template"], "source")?;
        let completion = self.tasks.verified_results.get(&id);
        if source == "external" || source == "both" {
            require(completion.is_some(), "尚未收到可信来源成果，不能通过审核")?;
        }
        // Only an authenticated internal adapter may supply a monetary base.
        let base = completion.and_then(|c| c["base_cents"].as_i64());
        if t["price"]["kind"] == "proportional" {
            require(
                completion.is_some_and(|c| c["source"] == t["price"]["base_source"]),
                "比例基数必须来自定价指定的可信来源",
            )?;
        }
        let calculation = calculate(&t["price"], base)?;
        let reserved = t["reserved_cents"].clone();
        t["reserved_cents"] = calculation["amount_cents"].clone();
        let confirmation = json!({"id":id,"claim_id":id,"owner":t["owner"],"calculation":calculation,"previous_reservation_cents":reserved,"reservation_cents":t["reserved_cents"],"price":t["price"],"submission":t["receipt"]["id"],"confirmed_by":actor,"confirmed_at":now,"status":"pending_ledger","withdrawable":false,"external_payment":false});
        self.tasks
            .confirmations
            .insert(id.clone(), confirmation.clone());
        self.tasks.changes.push(json!({"type":"commission_confirmed","claim_id":id,"at":now,"actor":actor,"confirmation":confirmation}));
        t["status"] = json!("approved");
        Ok(())
    }
    pub(super) fn review_command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        now: i64,
    ) -> Result<Value> {
        if action == "review_batch_preview" || action == "review_batch" {
            let entries = array(input, "entries")?;
            require(
                !entries.is_empty() && entries.len() <= 50,
                "批量操作须选1至50条",
            )?;
            let mut unique = std::collections::BTreeSet::new();
            let mut total = 0_i128;
            for e in entries {
                let id = text(e, "id")?;
                require(unique.insert(id), "批量条目重复")?;
                let t = self.tasks.instances.get(id).ok_or(missing())?;
                require(
                    t["status"] == "reviewing"
                        && t["reviewer"] == actor
                        && t["version"] == e["version"],
                    "批量条目状态、经办或版本已改变",
                )?;
                require(
                    t["template"]["double_review"] == false
                        && t["template"]["source"] == "manual"
                        && t["price"]["kind"] == "fixed",
                    "高风险、外部成果或比例佣金须单独核查",
                )?;
                total += i128::from(number(&calculate(&t["price"], None)?, "amount_cents")?);
            }
            require(total <= 9_007_199_254_740_991, "批量总数超出安全整数")?;
            if action == "review_batch_preview" {
                let preview = json!({"id":generated,"kind":"batch","actor":actor,"entries":entries,"amount_cents":total as i64,"created_at":now});
                self.tasks
                    .proposals
                    .insert(generated.into(), preview.clone());
                return Ok(preview);
            }
            let preview = self
                .tasks
                .proposals
                .get(text(input, "preview_id")?)
                .ok_or(missing())?;
            require(
                preview["kind"] == "batch"
                    && preview["actor"] == actor
                    && preview["entries"] == input["entries"]
                    && input["confirmed"] == true,
                "批量内容必须与确认预览完全相同",
            )?;
            // Service transaction restores all changes on failure: no partial batch confirmation.
            for e in entries {
                self.review_command(actor,"review_decide",&json!({"id":e["id"],"version":e["version"],"decision":"approve","reason":text(input,"reason")?,"checklist_confirmed":true}),generated,now)?;
            }
            return Ok(json!({"count":entries.len(),"amount_cents":total as i64}));
        }
        let id = text(input, "id")?;
        let mut t = self.tasks.instances.get(id).cloned().ok_or(missing())?;
        if input["version"].as_u64() != Some(version(&t)) {
            return Err(conflict());
        }
        if action == "evidence_reveal" {
            text(input, "reason")?;
            require(
                t["reviewer"] == actor
                    && matches!(
                        t["status"].as_str(),
                        Some("reviewing" | "awaiting_verification")
                    )
                    || self
                        .authorize(actor, "verifier", text(&t, "product_id")?)
                        .is_ok(),
                "敏感证据仅限本单审核经办或授权复核人员",
            )?;
            return Ok(json!({"id":id,"submissions":t["submissions"]}));
        }
        match action {
            "review_claim" => {
                require(t["status"] == "submitted", "该任务已被领取或不在待审状态")?;
                require(t["owner"] != actor, "不得审核本人任务")?;
                t["status"] = json!("reviewing");
                t["reviewer"] = json!(actor);
                t["review_claimed_at"] = json!(now);
            }
            "review_decide" => {
                require(
                    t["status"] == "reviewing" && t["reviewer"] == actor,
                    "只允许独占经办提交本单审核结论",
                )?;
                text(input, "reason")?;
                let decision = text(input, "decision")?;
                match decision {
                    "approve" => {
                        require(
                            input["checklist_confirmed"] == true,
                            "请逐项核对模板完成条件、证据、订单去重和规则版本",
                        )?;
                        if t["template"]["double_review"] == true {
                            t["status"] = json!("awaiting_verification");
                            t["first_reviewer"] = json!(actor);
                        } else {
                            self.approve_task(&mut t, actor, now)?;
                        }
                    }
                    "reject" => {
                        text(input, "reason_code")?;
                        t["status"] = json!("rejected");
                        t["reservation_released"] =
                            json!(t["template"]["release_on_terminal"] == true);
                    }
                    "supplement" => {
                        let count = number(&t, "supplement_count")?;
                        require(
                            count < number(&t["template"], "max_supplements")?,
                            "补交次数已达模板限制",
                        )?;
                        let fields = array(input, "fields")?;
                        require(
                            !fields.is_empty()
                                && fields.iter().all(|id| {
                                    array(&t["template"], "fields")
                                        .is_ok_and(|schema| schema.iter().any(|f| f["id"] == *id))
                                }),
                            "须列出模板内明确的缺失字段",
                        )?;
                        t["status"] = json!("supplement");
                        t["supplement_fields"] = input["fields"].clone();
                        t["supplement_count"] = json!(count + 1);
                        t["supplement_until"] = json!(next_time(
                            now,
                            positive(&t["template"], "supplement_seconds")?
                        )?);
                        t["submit_until"] = json!(
                            t["submit_until"]
                                .as_i64()
                                .unwrap_or(0)
                                .max(number(&t, "supplement_until")?)
                        );
                        t["preview"] = Value::Null;
                    }
                    _ => return Err(invalid("审核结论错误")),
                }
                let submission = t["receipt"]["id"].clone();
                t["decisions"].as_array_mut().unwrap().push(json!({"actor":actor,"at":now,"decision":decision,"reason":input["reason"],"reason_code":input["reason_code"],"submission_id":submission,"checklist_confirmed":input["checklist_confirmed"]}));
            }
            "review_verify" => {
                require(
                    t["status"] == "awaiting_verification"
                        && t["first_reviewer"] != actor
                        && t["owner"] != actor,
                    "须由另一名复核人员处理高风险结论",
                )?;
                text(input, "reason")?;
                require(input["approve"].is_boolean(), "请选择复核结论")?;
                if input["approve"] == true {
                    require(input["checklist_confirmed"] == true, "请确认复核检查清单")?;
                    self.approve_task(&mut t, actor, now)?;
                } else {
                    t["status"] = json!("submitted");
                    t["reviewer"] = Value::Null;
                }
                t["decisions"].as_array_mut().unwrap().push(json!({"actor":actor,"at":now,"decision":"verification","approve":input["approve"],"reason":input["reason"]}));
            }
            "task_repair_propose" | "review_revoke_propose" => {
                text(input, "reason")?;
                text(input, "evidence")?;
                if action == "task_repair_propose" {
                    require(
                        matches!(
                            t["status"].as_str(),
                            Some("claimed" | "in_progress" | "paused")
                        ) && !self.tasks.confirmations.contains_key(id),
                        "修复仅限未审核、未确认佣金的任务",
                    )?;
                    require(
                        matches!(
                            input["new_status"].as_str(),
                            Some("claimed" | "in_progress")
                        ),
                        "修复不得绕过提交或审核",
                    )?;
                } else {
                    require(t["status"] == "approved", "仅可申请撤销已通过的结论")?;
                }
                let p = json!({"id":generated,"kind":action,"task_id":id,"actor":actor,"task_version":t["version"],"old_status":t["status"],"new_status":input["new_status"],"reason":input["reason"],"evidence":input["evidence"],"status":"pending","at":now});
                self.tasks.proposals.insert(generated.into(), p.clone());
                return Ok(p);
            }
            "task_repair_review" | "review_revoke_review" => {
                let pid = text(input, "proposal_id")?;
                let mut p = self.tasks.proposals.get(pid).cloned().ok_or(missing())?;
                require(
                    p["task_id"] == id
                        && p["actor"] != actor
                        && p["status"] == "pending"
                        && p["task_version"] == t["version"],
                    "更正须独立审核且原任务版本不可改变",
                )?;
                require(
                    p["kind"]
                        == if action == "task_repair_review" {
                            "task_repair_propose"
                        } else {
                            "review_revoke_propose"
                        },
                    "更正类型不符",
                )?;
                text(input, "reason")?;
                require(input["approve"].is_boolean(), "请选择更正复核结论")?;
                if input["approve"] == true {
                    if action == "task_repair_review" {
                        t["status"] = p["new_status"].clone();
                        if t["status"] == "in_progress" && t["started_at"].is_null() {
                            t["started_at"] = json!(now);
                        }
                    } else {
                        t["status"] = json!("disputed");
                        // Preserve the original confirmation; an append-only hold requests downstream reconciliation.
                        self.tasks.changes.push(json!({"type":"confirmation_disputed","claim_id":id,"proposal_id":pid,"actor":actor,"at":now,"external_clawback":false,"requires_ledger_reconciliation":true}));
                    }
                }
                p["status"] = json!(if input["approve"] == true {
                    "approved"
                } else {
                    "rejected"
                });
                p["reviewer"] = json!(actor);
                p["review_reason"] = input["reason"].clone();
                p["reviewed_at"] = json!(now);
                self.tasks.proposals.insert(pid.into(), p);
            }
            "quality_inspect" => {
                require(!array(&t, "decisions")?.is_empty(), "尚无审核结论可抽检")?;
                text(input, "reason")?;
                require(input["consistent"].is_boolean(), "请选择抽检一致性结论")?;
                self.tasks.inspections.push(json!({"id":generated,"task_id":id,"actor":actor,"at":now,"consistent":input["consistent"],"reason":input["reason"],"task_version":t["version"],"submission":t["receipt"]["id"],"next_action":if input["consistent"]==false{"request_independent_correction"}else{"none"}}));
            }
            _ => return Err(invalid("未知审核动作")),
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
