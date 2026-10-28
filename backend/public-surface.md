Crowd domains expose remote adapters and south contracts. Only catalog and tasks
also expose their existing verified event adapters. Internal modules are private.

```compile_fail
use crowd_api::identity::domain;
```
```compile_fail
use crowd_api::identity::north::local;
```
```compile_fail
use crowd_api::identity::south::adapter;
```
```compile_fail
use crowd_api::identity::south::port::DomainPort;
```
```compile_fail
use crowd_api::platform::south::port::State;
```
```compile_fail
use crowd_api::platform::south::port::Repository;
```
```compile_fail
use crowd_api::receipts::event;
```
```no_run
use crowd_api::{
    identity::{remote, south::port::IdentityProvider},
    catalog::event::RecordVerifiedCompletion,
    tasks::event::RecordVerifiedTaskResult,
};
fn attach_trusted_source(pool: sqlx::PgPool) {
    let _completion = RecordVerifiedCompletion::new(pool.clone());
    let _task_result = RecordVerifiedTaskResult::new(pool);
}
```
