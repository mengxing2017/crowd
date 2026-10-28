# ADR 0014：收紧领域入口，仅保留实际事件处理

日期：2026-10-29。状态：代码已实施。接续 [ADR 0013](0013-domain-north-south-ports.md)。

## 对外入口

每个业务领域只公开 `remote` 与 `south`；只有已经处理产品完成、任务成果事件的 `catalog`、`tasks` 额外公开 `event`。

- `remote`：重新导出本领域 HTTP 适配入口，HTTP 路径保持兼容。
- `south/port`：明确提供方的接口。进程内规则与服务契约为 crate 内可见，不开放为 Rust SDK。
- `catalog/event/RecordVerifiedCompletion`、`tasks/event/RecordVerifiedTaskResult`：承接原有完成／成果记录服务，保留来源、事件编号、时间及去重检查。构造函数接收连接池，仓储留在内部。

没有事件需求的领域不创建 event 目录、空处理器或通用事件占位。没有新增事件总线、消息队列、路由、消费者注册、支付回调或自动付款。

## 内部实现

`domain`、`north`、`north/local` 和 `south/adapter` 的可见性限制在所属领域，其他领域即便同属当前 crate，也无法直接访问。领域根模块重新导出 remote，而不公开整个 north。调用方通过提供方 south 的规则或服务契约连接；consumer `DomainPort` 不再对 crate 外公开。

全局 `State` 和 `Repository` 不再是公开接口。开发启动通过明确的 `platform/south/port/bootstrap_staff` 接口初始化管理员，不暴露事务仓储或状态文档。根 `ApplicationServices` 与 HTTP 状态中的服务字段仅在 crate 内可见。

历史 Service 委派只保留为测试夹具，不进入生产构建；community 命名空间也只用于内部历史记录兼容测试。持久化序列化兼容继续验证，原 Rust 内部路径不再作为外部兼容接口。HTTP／TypeScript 契约没有变化。

## 测试与调用迁移

白盒业务测试通过 `tests/mod.rs` 作为 crate 单元测试编译，内部规则由 south 测试契约访问，不为测试重新开放 domain。共用任务夹具只加载一次。数据库测试仍标记为 ignored，由 `make test-business` 使用隔离嵌入式 PostgreSQL 执行；runner 改为 `cargo test --lib <测试模块> -- --ignored --nocapture --test-threads=1`，保持原两条数据库流程。

`public_surface` 检查所有领域公开入口及内部可见性，并拒绝无需求的事件目录。Rust 文档编译失败测试验证外部调用者不能导入 domain、north/local、adapter、DomainPort、State 或 Repository，同时验证真实 remote／south／event 接口可以导入。已有依赖边界、模块入口和业务回归测试继续执行。

不改变事务锁、幂等键、资料快照、佣金数字、人工处理状态或迁移历史。自动化验证不等于业务验收。

数据库 runner 显式按 community_postgres、finance_postgres 顺序启动，保留前者对空临时数据库的要求；每个流程内部仍验证并发操作。
