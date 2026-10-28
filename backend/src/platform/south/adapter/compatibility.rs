use crate::identity::south::port::rules::User;
use crate::platform::south::port::State;
use crate::shared::Result;
use serde_json::Value;
impl State {
    pub fn user(&self, id: &str) -> Result<&User> {
        crate::identity::south::port::rules::BehaviorRules::user(self, id)
    }
    pub fn authorize(&self, actor: &str, role: &str, target: &str) -> Result<()> {
        crate::identity::south::port::rules::BehaviorRules::authorize(self, actor, role, target)
    }
    pub fn register_user(&mut self, id: &str, agreement: Value, source: Option<String>) {
        crate::identity::south::port::rules::BehaviorRules::register_user(
            self, id, agreement, source,
        )
    }
    #[cfg(test)]
    pub fn register(
        &mut self,
        id: &str,
        code: &str,
        agreement: Value,
        source: Option<String>,
    ) -> Result<()> {
        crate::promotion::south::port::rules::BehaviorRules::register(
            self, id, code, agreement, source,
        )
    }
    pub fn resolve(&self, code: &str) -> Result<Value> {
        crate::promotion::south::port::rules::BehaviorRules::resolve(self, code)
    }
    pub fn log(&mut self, actor: &str, action: &str, target: &str, request: &str, time: &str) {
        crate::audit::south::port::rules::BehaviorRules::log(
            self, actor, action, target, request, time,
        )
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
        crate::audit::south::port::rules::BehaviorRules::log_failure(
            self, actor, action, target, request, time, code,
        )
    }
    #[cfg(test)]
    pub fn catalog_command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        now: i64,
    ) -> Result<Value> {
        crate::catalog::south::port::rules::CommandsRules::catalog_command(
            self, actor, action, input, generated, now,
        )
    }
    pub fn catalog_read(
        &mut self,
        actor: &str,
        section: &str,
        target: &str,
        now: i64,
    ) -> Result<Value> {
        crate::catalog::south::port::rules::QueriesRules::catalog_read(
            self, actor, section, target, now,
        )
    }
    pub fn visible_release(&self, id: &str, now: i64) -> Result<Value> {
        crate::catalog::south::port::rules::QueriesRules::visible_release(self, id, now)
    }
    pub fn public_catalog(&self, query: &Value, now: i64) -> Result<Value> {
        crate::catalog::south::port::rules::QueriesRules::public_catalog(self, query, now)
    }
    pub fn catalog_access(&self, actor: &str, action: &str, input: &Value) -> Result<()> {
        crate::catalog::south::port::rules::AccessRules::catalog_access(self, actor, action, input)
    }
    pub fn reconcile_lifecycle(&mut self, now: i64) {
        crate::catalog::south::port::rules::LifecycleRules::reconcile_lifecycle(self, now)
    }
    pub fn collect_touch(&mut self, input: &Value, generated: &str, now: i64) -> Result<Value> {
        crate::catalog::south::port::rules::AttributionRules::collect_touch(
            self, input, generated, now,
        )
    }
    pub fn bind_touch(&mut self, actor: &str, token: &str, now: i64) -> Result<Value> {
        crate::catalog::south::port::rules::AttributionRules::bind_touch(self, actor, token, now)
    }
    #[cfg(test)]
    pub fn tasks_command(
        &mut self,
        actor: &str,
        action: &str,
        input: &Value,
        generated: &str,
        now: i64,
    ) -> Result<Value> {
        crate::tasks::south::port::rules::CommandsRules::tasks_command(
            self, actor, action, input, generated, now,
        )
    }
    pub fn tasks_read(&self, actor: &str, section: &str, target: &str, now: i64) -> Result<Value> {
        crate::tasks::south::port::rules::QueriesRules::tasks_read(
            self, actor, section, target, now,
        )
    }
    pub fn tasks_access(&self, actor: &str, action: &str, input: &Value) -> Result<()> {
        crate::tasks::south::port::rules::AccessRules::tasks_access(self, actor, action, input)
    }
    pub fn task_snapshot(&self, actor: &str, task: &Value, now: i64) -> Result<Option<Value>> {
        crate::tasks::south::port::rules::TaskDefinitionRules::task_snapshot(self, actor, task, now)
    }
    pub fn attach_task(&mut self, claim: &Value, snapshot: Option<Value>, now: i64) -> Result<()> {
        crate::tasks::south::port::rules::TaskDefinitionRules::attach_task(
            self, claim, snapshot, now,
        )
    }
    pub fn validate_task_reference(&self, task: &mut Value, start: i64, end: i64) -> Result<()> {
        crate::tasks::south::port::rules::TaskLifecycleRules::validate_task_reference(
            self, task, start, end,
        )
    }
    pub fn expire_tasks(&mut self, now: i64) -> usize {
        crate::tasks::south::port::rules::TaskLifecycleRules::expire_tasks(self, now)
    }
}
