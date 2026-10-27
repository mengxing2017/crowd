use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Failure(pub &'static str, pub &'static str);
pub type Result<T> = std::result::Result<T, Failure>;
pub fn invalid(message: &'static str) -> Failure {
    Failure("INVALID_INPUT", message)
}
pub fn text<'a>(v: &'a Value, field: &str) -> Result<&'a str> {
    v[field]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 4096)
        .ok_or(invalid("必填字段缺失或过长"))
}
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct State {
    #[serde(default)]
    pub catalog: crate::catalog::domain::Catalog,
    pub users: BTreeMap<String, User>,
    pub rules: Vec<Value>,
    pub codes: BTreeMap<String, Code>,
    pub cases: Vec<Value>,
    pub audit: Vec<Value>,
    pub idempotency: BTreeMap<String, Value>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub nickname: String,
    pub avatar: Option<String>,
    pub version: u64,
    pub status: String,
    pub roles: Vec<String>,
    pub scope: Vec<String>,
    pub receipts: Vec<Value>,
    pub restrictions: Vec<Value>,
    pub agreement: Value,
    pub source: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Code {
    pub owner: String,
    pub version: u64,
    pub active: bool,
    pub product: Option<String>,
    pub task: Option<String>,
}
impl State {
    pub fn user(&self, id: &str) -> Result<&User> {
        self.users
            .get(id)
            .filter(|u| u.status != "closed")
            .ok_or(Failure("SESSION_EXPIRED", "请重新登录"))
    }
    pub fn authorize(&self, actor: &str, role: &str, target: &str) -> Result<()> {
        let u = self.user(actor)?;
        if u.status == "disabled" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号已停用，请联系客服申诉"));
        }
        if u.roles.iter().any(|r| r == role) && (u.scope.iter().any(|s| s == "*" || s == target)) {
            Ok(())
        } else {
            Err(Failure("FORBIDDEN", "无此操作权限或数据范围"))
        }
    }
    pub fn log(&mut self, actor: &str, action: &str, target: &str, request: &str, time: &str) {
        let published: Vec<Value> = self
            .rules
            .iter()
            .filter(|r| r["status"] == "published")
            .map(|r| json!({"id":r["id"],"kind":r["kind"],"version":r["version"]}))
            .collect();
        self.audit.push(json!({"published_rules":published,"actor":actor,"action":action,"object":target,"request_id":request,"time":time,"result":"success","rule_version":"M01-M04-v1"}));
    }
    pub fn log_failure(
        &mut self,
        actor: &str,
        action: &str,
        target: &str,
        request: &str,
        time: &str,
        code: &str,
    ) {
        self.log(actor, action, target, request, time);
        if let Some(last) = self.audit.last_mut() {
            last["result"] = json!(code);
        }
    }
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
    pub fn register_user(&mut self, id: &str, agreement: Value, source: Option<String>) {
        if self.users.contains_key(id) {
            return;
        }
        self.users.insert(
            id.into(),
            User {
                id: id.into(),
                nickname: "新用户".into(),
                avatar: None,
                version: 1,
                status: "active".into(),
                roles: vec!["user".into()],
                scope: vec![id.into()],
                receipts: vec![],
                restrictions: vec![],
                agreement,
                source,
            },
        );
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
    // The published M01 product/task relation is the authoritative initial target catalogue.
    // A future product module must implement this same validation boundary before replacing it.
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
    pub fn read(
        &mut self,
        actor: &str,
        section: &str,
        target: &str,
        request: &str,
        time: &str,
    ) -> Result<Value> {
        self.user(actor)?;
        match section {
            "me" => { let mut value=serde_json::to_value(self.user(actor)?).expect("serialize user");value["receipts"]=json!(self.user(actor)?.receipts.iter().map(mask_receipt).collect::<Vec<_>>());Ok(value) },
            "codes" => Ok(json!(self.codes.iter().filter(|(_,c)| c.owner == actor).map(|(id,c)|json!({"id":id,"code":c})).collect::<Vec<_>>())),
            "rules" => { self.authorize(actor,"rule_manager","platform")?; Ok(json!(self.rules)) },
            "users" => { self.authorize(actor,"identity_reviewer",target)?; let u=self.user(target)?; Ok(json!({"id":u.id,"status":u.status,"version":u.version,"roles":u.roles,"scope":u.scope,"restrictions":u.restrictions})) },
            "audit" => { self.authorize(actor,"auditor",target)?; Ok(json!(self.audit.iter().filter(|a| target == "*" || a["object"] == target).rev().take(100).collect::<Vec<_>>())) },
            "command_result" => { let previous=self.idempotency.get(&format!("{actor}:{target}")).ok_or(Failure("RESULT_NOT_FOUND","未查询到已提交结果，请使用原请求键重试"))?;self.check_command_access(actor,previous["action"].as_str().unwrap_or(""),&previous["input"])?;Ok(previous["result"].clone()) },
            "managed_codes" => {self.authorize(actor,"promotion_manager",target)?; Ok(json!(self.codes.iter().filter(|(_,c)| c.owner==target).map(|(id,c)|json!({"id":id,"code":c})).collect::<Vec<_>>()))},
            "cases" => Ok(json!(self.cases.iter().filter(|v| v["owner"] == actor).collect::<Vec<_>>())),
            "receipts" => { if target != actor { return Err(Failure("FORBIDDEN","后台查看收款资料必须关联申请范围与用途，申请服务尚未接入")); } Ok(json!(self.user(actor)?.receipts.iter().map(mask_receipt).collect::<Vec<_>>())) },
            _ => Err(Failure("NOT_FOUND","接口不存在")),
        }.inspect(|_| { self.log(actor,section,target,request,time); })
    }
    pub fn check_command_access(&self, actor: &str, action: &str, input: &Value) -> Result<()> {
        if crate::catalog::domain::is_command(action) {
            return self.catalog_access(actor, action, input);
        }
        let fields = match action {
            "profile" => Some(&["nickname", "avatar", "version"][..]),
            "receipt" => Some(
                &[
                    "method",
                    "account",
                    "name",
                    "file_id",
                    "confirmed",
                    "version",
                ][..],
            ),
            "receipt_reveal" => Some(&["id", "reason"][..]),
            _ => None,
        };
        if let Some(fields) = fields {
            let object = input.as_object().ok_or(invalid("请求内容必须为对象"))?;
            if object.keys().any(|key| !fields.contains(&key.as_str())) {
                return Err(invalid("请求包含当前操作不接受的字段"));
            }
        }
        let user = self.user(actor)?;
        if user.status == "disabled" && action != "logout" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号已停用，请联系客服申诉"));
        }
        match action {
            "rule_save" | "rule_publish" | "rule_disable" => {
                self.authorize(actor, "rule_manager", "platform")
            }
            "access" | "restriction" => {
                self.authorize(actor, "identity_reviewer", text(input, "user_id")?)
            }
            "code_rotate" => {
                let code = self
                    .codes
                    .get(text(input, "id")?)
                    .ok_or(Failure("NOT_FOUND", "推广码不存在"))?;
                self.authorize(actor, "promotion_manager", &code.owner)
            }
            _ => Ok(()),
        }
    }
    pub fn command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        request: &str,
        time: &str,
    ) -> Result<Value> {
        let u = self.user(actor)?;
        if u.status == "disabled" {
            return Err(Failure("ACCOUNT_RESTRICTED", "账号已停用，请联系客服申诉"));
        }
        let result = match action {
            "profile" => {
                let nickname = text(input, "nickname")?;
                let u = self.users.get_mut(actor).unwrap();
                check_version(u.version, input)?;
                u.nickname = nickname.into();
                u.avatar = input["avatar"].as_str().map(str::to_owned);
                u.version += 1;
                json!({"version":u.version})
            }
            "receipt" => {
                let method = text(input, "method")?;
                if !["wechat", "alipay"].contains(&method) {
                    return Err(invalid("收款方式无效"));
                }
                if input["confirmed"] != true {
                    return Err(invalid("请主动确认资料可供本人收款"));
                }
                let mut record = json!({"id":generated,"method":method,"status":"confirmed","time":time,"owner":actor,"official_verified":false});
                if method == "wechat" {
                    record["file_id"] = json!(text(input, "file_id")?);
                } else {
                    // Supported formats remain unconfirmed; accepting opaque drafts avoids declaring validity.
                    record["account"] = json!(text(input, "account")?);
                    record["name"] = input["name"].clone();
                    record["status"] = json!("draft");
                }
                let u = self.users.get_mut(actor).unwrap();
                check_version(u.version, input)?;
                record["version"] = json!(u.version + 1);
                u.receipts.push(record);
                u.version += 1;
                json!({"id":generated,"version":u.version,"notice":"资料已保存，不代表官方账户验证；支付宝支持形式待确认"})
            }
            "receipt_disable" => {
                let id = text(input, "id")?;
                let u = self.users.get_mut(actor).unwrap();
                if u.receipts.iter().any(|r| r["disables"] == id) {
                    return Ok(json!({"version":u.version}));
                }
                check_version(u.version, input)?;
                // Append status event; original receipt and any snapshot never change.
                if !u
                    .receipts
                    .iter()
                    .any(|r| r["id"] == id && r["method"].is_string())
                {
                    return Err(Failure("NOT_FOUND", "资料不存在"));
                }
                u.receipts
                    .push(json!({"id":generated,"disables":id,"status":"disabled","time":time}));
                u.version += 1;
                json!({"version":u.version})
            }
            "code_create" => {
                let product = input["product"].as_str();
                let task = input["task"].as_str();
                self.validate_target(product, task)?;
                if self
                    .user(actor)?
                    .restrictions
                    .iter()
                    .any(|r| r["scope"] == "promotion")
                {
                    return Err(Failure(
                        "ACCOUNT_RESTRICTED",
                        "推广功能受限，请联系客服申诉",
                    ));
                }
                if let Some((id, _)) = self.codes.iter().find(|(_, c)| {
                    c.owner == actor
                        && c.active
                        && c.product.as_deref() == product
                        && c.task.as_deref() == task
                }) {
                    return Ok(json!({"id":id}));
                }
                if self.codes.contains_key(generated) {
                    return Err(Failure("CODE_COLLISION", "推广码冲突，请重试"));
                }
                self.codes.insert(
                    generated.into(),
                    Code {
                        owner: actor.into(),
                        version: 1,
                        active: true,
                        product: product.map(str::to_owned),
                        task: task.map(str::to_owned),
                    },
                );
                json!({"id":generated})
            }
            "code_rotate" => {
                let id = text(input, "id")?;
                let reason = text(input, "reason")?;
                let approval = text(input, "approval")?;
                let c = self
                    .codes
                    .get(id)
                    .ok_or(Failure("NOT_FOUND", "推广码不存在"))?
                    .clone();
                self.authorize(actor, "promotion_manager", &c.owner)?;
                if !c.active {
                    return Err(Failure("VERSION_CONFLICT", "原推广码已经撤销"));
                }
                check_version(c.version, input)?;
                if self.codes.contains_key(generated) {
                    return Err(Failure("CODE_COLLISION", "推广码冲突，请重试"));
                }
                self.codes.get_mut(id).unwrap().active = false;
                self.codes.insert(
                    generated.into(),
                    Code {
                        version: c.version + 1,
                        ..c
                    },
                );
                self.log(actor, "REQ-04-008", id, request, time);
                self.audit.last_mut().unwrap()["reason"] = json!(reason);
                self.audit.last_mut().unwrap()["approval"] = json!(approval);
                self.audit.last_mut().unwrap()["approved_by"] = json!(actor);
                json!({"id":generated})
            }
            "rule_save" => {
                self.authorize(actor, "rule_manager", "platform")?;
                let kind = text(input, "kind")?;
                if ![
                    "business_chain",
                    "operator",
                    "roles",
                    "completion",
                    "review",
                    "product_tasks",
                    "attribution",
                    "digital_boundary",
                    "amount_unit",
                    "release_scope",
                    "agreements",
                ]
                .contains(&kind)
                {
                    return Err(invalid("规则类型无效"));
                }
                if !input["content"].is_object() {
                    return Err(invalid("规则内容必须为对象"));
                }
                let version = self.rules.iter().filter(|r| r["kind"] == kind).count() as u64;
                check_version(version, input)?;
                self.rules.push(json!({"id":generated,"kind":kind,"version":version+1,"status":"draft","content":input["content"],"owner":actor,"time":time}));
                json!({"id":generated,"version":version+1})
            }
            "rule_publish" | "rule_disable" => {
                self.authorize(actor, "rule_manager", "platform")?;
                let id = text(input, "id")?;
                let pos = self
                    .rules
                    .iter()
                    .position(|r| r["id"] == id)
                    .ok_or(Failure("NOT_FOUND", "规则版本不存在"))?;
                let r = self.rules[pos].clone();
                check_version(r["version"].as_u64().unwrap(), input)?;
                let expected = if action == "rule_publish" {
                    "draft"
                } else {
                    "published"
                };
                if r["status"] != expected {
                    return Err(Failure("VERSION_CONFLICT", "规则状态已改变"));
                }
                if action == "rule_publish" {
                    if r["kind"] == "digital_boundary" && r["content"]["manual_only"] != true {
                        return Err(invalid("仅支持数字记录与人工线下结算"));
                    }
                    if r["kind"] == "product_tasks" {
                        validate_catalogue(&r["content"])?;
                    }
                    if r["kind"] == "agreements" {
                        text(&r["content"], "service")?;
                        text(&r["content"], "privacy")?;
                    }
                    for other in &mut self.rules {
                        if other["kind"] == r["kind"] && other["status"] == "published" {
                            other["status"] = json!("disabled");
                        }
                    }
                }
                self.rules[pos]["status"] = json!(if action == "rule_publish" {
                    "published"
                } else {
                    "disabled"
                });
                json!({"id":id,"status":self.rules[pos]["status"]})
            }
            "access" | "restriction" => {
                let target = text(input, "user_id")?;
                self.authorize(actor, "identity_reviewer", target)?;
                if target == actor {
                    return Err(Failure("FORBIDDEN", "不能修改自己的权限或限制"));
                }
                text(input, "reason")?;
                let u = self
                    .users
                    .get_mut(target)
                    .ok_or(Failure("NOT_FOUND", "用户不存在"))?;
                check_version(u.version, input)?;
                if action == "access" {
                    let roles: Vec<String> = serde_json::from_value(input["roles"].clone())
                        .map_err(|_| invalid("角色格式错误"))?;
                    if roles.iter().any(|r| {
                        ![
                            "user",
                            "rule_manager",
                            "identity_reviewer",
                            "promotion_manager",
                            "auditor",
                            "operator",
                            "product_reviewer",
                            "product_emergency",
                            "attribution_reviewer",
                            "task_reviewer",
                            "settlement_reviewer",
                            "manual_handler",
                            "verifier",
                            "support",
                        ]
                        .contains(&r.as_str())
                    }) {
                        return Err(invalid("未知角色"));
                    }
                    u.roles = roles;
                    u.scope = serde_json::from_value(input["scope"].clone())
                        .map_err(|_| invalid("数据范围格式错误"))?;
                } else {
                    let scope = text(input, "restriction_scope")?;
                    if !["all", "withdrawal", "promotion"].contains(&scope) {
                        return Err(invalid("限制范围无效"));
                    }
                    u.restrictions.push(json!({"scope":scope,"reason":input["reason"],"appeal":"联系客服","time":time}));
                    if scope == "all" {
                        u.status = "disabled".into();
                    }
                }
                u.version += 1;
                json!({"version":u.version})
            }
            "correction" | "closure" | "merge" => {
                text(input, "reason")?;
                // No account deletion or identity merge before dependent ledgers can be checked.
                let status = if action == "closure" || action == "merge" {
                    "pending_dependency_check"
                } else {
                    "pending_verification"
                };
                self.cases.push(json!({"id":generated,"owner":actor,"kind":action,"status":status,"reason":input["reason"],"affected_application":input["application_id"],"time":time}));
                json!({"id":generated,"status":status})
            }
            "phone" => {
                return Err(Failure(
                    "FEATURE_NOT_ENABLED",
                    "手机号绑定策略及可信验证渠道尚未启用，原绑定保持不变",
                ));
            }
            _ => return Err(Failure("NOT_FOUND", "操作不存在")),
        };
        self.log(actor, action, actor, request, time);
        Ok(result)
    }
}
pub fn check_version(version: u64, input: &Value) -> Result<()> {
    if input["version"].as_u64() != Some(version) {
        Err(Failure("VERSION_CONFLICT", "版本已变更，请重新查询"))
    } else {
        Ok(())
    }
}
fn mask_receipt(v: &Value) -> Value {
    let mut v = v.clone();
    if let Some(s) = v["account"].as_str() {
        let chars: Vec<char> = s.chars().collect();
        let tail: String = chars.iter().rev().take(2).rev().collect();
        v["account"] = json!(format!("***{tail}"));
    }
    v.as_object_mut().unwrap().remove("name");
    v
}

fn validate_catalogue(content: &Value) -> Result<()> {
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

/// Integer-only display; no floating point participates in business amount decisions.
pub fn display_cents(cents: i64) -> String {
    let value = i128::from(cents);
    let absolute = value.abs();
    format!(
        "{}{}.{:02}",
        if value < 0 { "-" } else { "" },
        absolute / 100,
        absolute % 100
    )
}
