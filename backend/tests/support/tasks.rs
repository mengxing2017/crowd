use crowd_api::community::domain::State;
use serde_json::{Value, json};
pub fn price(now: i64) -> Value {
    json!({"task_id":"task","settlement_scope":"test-scope","currency":"CNY","kind":"fixed","amount_cents":101,"starts_at":now-100,"ends_at":now+10000})
}
pub fn template(now: i64) -> Value {
    json!({"name":"完成业务操作","task_type":"registration","completion":"按步骤提供真实说明","failure_conditions":"伪造证据不通过","review_goal":"核对业务事实","source":"manual","steps":[{"id":"one","title":"完成第一步","required":true},{"id":"two","title":"核对结果","required":true}],"fields":[{"id":"description","kind":"text","label":"成果说明","purpose":"人工审核","required":true,"max_length":500},{"id":"order","kind":"order","label":"订单编号","purpose":"去重核对","required":false,"source":"test","exclusive_scope":"business"},{"id":"photo","kind":"image","label":"证明图片","purpose":"核对执行结果","required":false}],"regions":[],"qualifications":[],"execution_seconds":1000,"submit_seconds":2000,"claim_until":now+5000,"total_quota":5,"per_user_quota":2,"participation":"repeat","allow_abandon":true,"release_on_terminal":true,"allow_withdraw":true,"pause_policy":"extend","double_review":false,"max_supplements":2,"supplement_seconds":200,"review_sla_seconds":100,"max_evidence_files":3,"max_file_bytes":1048576})
}
pub fn seed(s: &mut State, editor: &str, verifier: &str, now: i64) {
    for (kind, c) in [("price", price(now)), ("template", template(now))] {
        s.tasks_command(
            editor,
            &format!("{kind}_save"),
            &json!({"id":kind,"version":0,"content":c}),
            "ignored",
            now,
        )
        .unwrap();
        s.tasks_command(
            verifier,
            &format!("{kind}_review"),
            &json!({"id":kind,"version":1,"approve":true,"reason":"测试独立审批"}),
            "ignored",
            now,
        )
        .unwrap();
    }
}
pub fn instance(s: &mut State, id: &str, owner: &str, now: i64) {
    let task = json!({"id":"task","template_id":"template","price_id":"price","commission_cents":101,"currency":"CNY"});
    let snapshot = s.task_snapshot(owner, &task, now).unwrap();
    let claim = json!({"id":id,"owner":owner,"product_id":"product","release_id":"release","locked_at":now,"status":"claimed","task":task,"product":{"name":"测试产品"}});
    s.attach_task(&claim, snapshot, now).unwrap();
    s.catalog.claims.insert(id.into(), claim);
}
