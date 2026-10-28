# ADR 0013：domain、north 的领域依赖隔离

日期：2026-10-28。状态：代码已实施。本决策接续 [ADR 0009](0009-community-domain-ownership.md)。

## 约束与实现

`domain` 和 `north` 不直接引用其他业务领域，包括类型导入、组合导入、重导出、全限定调用和相对路径。各领域在自己的 `south/port/` 声明所需接口，由 `south/adapter/` 完成领域间连接。HTTP 路径、TypeScript 契约和业务规则不因本次重构改变。

领域规则不再在全局 `State` 上实现，也不接受 `State` 参数：规则通过本领域的 `DomainPort` 获取本领域聚合及所需的外部查询数据。原有实体、状态校验、金额计算及权限判定仍在所属领域。共享状态仅是 PostgreSQL 适配层中的事务文档，位于 `platform/south/adapter/state.rs`。

跨领域的读操作通过只读查询集合或用户视图提供，不能取得其他领域聚合的可变引用。涉及变更时使用明确的端口操作：

| 消费领域 | 端口操作 | 提供的数据或变更 |
| --- | --- | --- |
| catalog | `task_snapshot`、`validate_task_reference`、`attach_task` | 任务规则校验、领取快照与实例创建 |
| promotion | `register_user`、`visible_release` | 身份注册、可见产品发布版本 |
| tasks | `update_claim_progress`、`record_catalog_notification`、`sync_commissions` | 领取进度、提醒与佣金数字同步 |
| commissions | 任务确认、申请占用只读查询 | 数字计算与争议来源核对 |
| withdrawals | `ledger_append`、`allocate_commissions`、`commission_source_disputed`、`validate_alipay` | 占用流水、来源分配、争议与资料规则校验 |
| settlements | `release_withdrawal`、`ledger_append`、`save_withdrawal_application`、`save_withdrawal_preview` | 人工处理结果、释放或结转、审核清单 |
| receipts | `append_receipt_event`、规则及引用只读查询 | 乐观版本检查、追加资料事件与清理保护 |
| files | `FileAccess`、证据上传及读取适配 | 证据归属、收款快照与凭证访问 |
| 各业务领域 | `AuditWriter` | 追加审计及本次操作的元数据，禁止直接修改审计集合 |

接口使用当前事务内的数据，不新增独立提交。当前采用同步端口调用，没有新增消息队列、异步事件或最终一致性窗口。若以后改用事件，需明确事件版本、幂等、失败恢复与事务提交策略。

## 应用层与兼容

应用服务按既有用例保留在本领域 `north/local`；需要状态参数的服务以本领域端口为约束。请求上下文是 `shared::CommandContext`，不再借用其他领域的 north 类型。数据库事务基础设施由各自 south 提供。

平台的路由组装、跨领域命令和查询分派、附件校验连接及历史 `Service` 委派迁入 `platform/south/adapter`，north 通过自己的 south 入口调用。`platform/domain` 和 `community` 仅保留历史 Rust 导出兼容；新领域规则不能借此获取全局状态。兼容的 `State` 方法实现在 south，委派至具体领域规则。

`shared` 仅承载通用错误、整数校验、脱敏函数与请求上下文，不承载其他领域的业务服务或全局状态。

## 一致性与范围

保留已有 JSON 序列化键、数据库锁、幂等键、失败回滚及原子保存。收款资料追加版本仍检查预期版本；旧申请快照保持不变；结果待核实继续保留占用；人工结算仍由人员在系统外完成。没有数据库 migration、支付接口或付款调度变更。

这次隔离的是代码依赖与调用边界。持久化仍使用原共享事务文档，部分查询数据仍为 JSON 集合；不宣称已拆成独立数据库、独立提交的聚合或微服务。

## 防止回归

`layer_dependencies` 使用 Rust 语法树检查所有 domain、north 文件，包括嵌套导入、别名和内联模块中的路径。南向适配器允许组装领域；规则层仍禁止引入 Axum 和 SQLx。模块入口检查支持可见性及条件编译，但只允许文件模块声明和公开导出。

`isolated_identity` 使用只实现身份端口的内存测试对象执行注册和授权，验证规则不要求平台 State。既有领域、HTTP、序列化和隔离 PostgreSQL 流程继续验证行为与历史兼容。

## 后续封装收紧

2026-10-29 后的模块可见性、Service 测试夹具迁移与实际事件入口见 [ADR 0014](0014-domain-public-entrypoints.md)。本页的兼容路径描述保留为上一阶段记录。
