# ADR 0008：local 应用服务按用例拆分

## 问题

`community/north/local.rs` 集中登录、资料、通用读写、上传及证据查看；`catalog/north/local.rs` 集中生命周期、目录、访问及完成事件；`tasks/north/local.rs` 又为同一个 `community::Service` 追加方法。只将这些 `impl Service` 移入多个文件不能建立应用服务的职责边界。

## 决策与组织

将全部三个 `local.rs` 替换为 `local/` 目录，入口 `mod.rs` 仅声明与重新导出。独立服务持有自己的注入依赖，除构造方法外提供一个公开异步 `execute` 入口。

| 所属模块 | 独立应用用例 |
| --- | --- |
| `community/north/local` | `StaffLogin`、`WechatLogin`、`RedeemSession`、`ReadAgreements`、`ResolvePromotion`、`UploadFile`、`ReadPrivateFile` |
| `catalog/north/local` | `ReadPublicCatalog`、`CollectTouch`、`ReadProductImage`、`ReconcileCatalog`、`RecordVerifiedCompletion`、无状态的 `ValidateProductFiles` |
| `tasks/north/local` | `ReadEvidenceFile`、`RecordVerifiedTaskResult` |
| 兼容协议事务管线 | `ReadResource`、`ExecuteCommand` |

`ReadResource` 与 `ExecuteCommand` 负责原协议所需的认证、事务、审计、幂等及失败恢复，具体领域命令处理移入各领域的 `local/commands.rs`。输入文件检查、领域分派及任务敏感查看的重复请求审计分别位于职责文件中，不再向聚合服务追加实现。

`application.rs` 中的 `ApplicationServices` 仅负责组装具体用例，复用同一 PostgreSQL 连接池。HTTP adapter 通过 `app.services` 调用具体用例；会话兑换也通过应用服务执行，不再直接访问 repository。后台生命周期任务直接使用 `ReconcileCatalog`。

系统模块此前已经使用 `north/local/` 目录，其技术性 `SystemService` 保持现有实现；本次没有将其重新合并进业务服务。

## 兼容与事务约束

- 原 `community::north::local::Service` 保留在 `compatibility.rs`，维持原构造方式和公开方法，方法仅创建并委托对应用例。目录与任务模块不再 `impl Service`。
- 保留 `catalog::north::local::is_command` 的既有导出路径，直接重新导出领域分类函数。
- 不修改 HTTP 路径、请求／响应结构、数据库迁移和业务规则。
- 保留权限检查先于有效幂等结果返回、请求键与载荷绑定、原结果查询、重复敏感查看留痕等行为。
- 文件校验、到期维护、命令执行、数字变更、有效幂等结果和审计仍处于原事务中。发生业务失败时恢复原状态快照并记录失败，不让批量审核中的前序成功项被部分保存。
- 可信成果和访问记录保留来源事件、时间范围、重复请求及冲突处理；不新增对外成果入口或付款接口。

此次完成应用服务实现与依赖接入拆分。聚合状态、共享 repository、通用协议事务管线和跨领域数据所有权仍保留原边界，不宣称完成后端领域解耦。

## 验证

```bash
devbox run -- make check-backend
devbox run -- make test-business
```

`application_boundaries.rs` 验证三个业务模块不再存在聚合 `local.rs`，单文件不混合应用服务，每个具体服务仅有一个公开异步执行入口，兼容门面不执行业务，HTTP 和后台任务不再调用旧聚合服务。既有模块入口测试继续检查 `mod.rs`。

领域和 HTTP 测试验证原行为；独立嵌入式 PostgreSQL 集成测试覆盖新具体服务与旧门面调用的一致性、并发与原键重试、文件授权、证据版本、任务审核和批量失败后的状态及数字恢复。

2026-10-28 实际验证：Devbox / Rust 1.97.1 下 `make check-backend` 的格式、编译、Clippy 及 50 项测试全部通过，模块入口检查覆盖 31 个 `mod.rs`。`make test-business` 使用隔离的嵌入式 PostgreSQL 17，1 项数据库集成测试通过，未连接外部数据库。本次未改动前端或小程序，未执行真机验证；这些检查不代表业务验收完成。
