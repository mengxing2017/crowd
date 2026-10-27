# ADR 0006：Rust 模块入口只声明与导出

## 决策

所有 `mod.rs` 仅保留文件模块声明和重新导出。类型、端口、函数、常量、业务实现及内联模块均放在具名文件中。通过显式重新导出保留原有调用路径，不要求调用方知道内部文件名。

## 实现组织

- `catalog/domain`：`model` 保存目录状态，`dispatch` 分类命令和查询，`access` 处理权限，`queries` 处理读取，`lifecycle` 处理生命周期与通知，`attribution` 处理访问触点，`commands` 执行命令，`completion` 实现完成事件端口，`validation` 保存内容和参数校验。
- `tasks/domain`：将原入口的状态、操作分类、权限、查询、命令和公共校验分别移入 `model`、`dispatch`、`access`、`queries`、`commands`、`validation`；保留原 `execution`、`rules`、`review` 职责，使用显式依赖代替父入口通配导入。
- 可信任务成果端口定义位于 `tasks/south/port/verified_task_result.rs`，状态变更实现位于 `tasks/domain/verified_result.rs`。端口入口不再执行业务规则。
- `tasks/north` 和 `tasks/south` 使用独立文件模块，替换原内联模块声明。
- 系统信息、系统用例、健康检查、系统信息 HTTP 接口和 PostgreSQL 就绪检查分别位于具名实现文件；身份提供者、完成记录者及就绪检查端口同样使用独立文件。

模块间共享的原私有辅助方法只开放到所属领域模块（`pub(super)`），不作为新的公共业务 API 导出。

## 兼容与边界

现有 `Catalog`、`Tasks`、`SystemInfo`、`SystemService`、`PostgresReadiness`、端口、规则计算和 HTTP handler 路径通过重新导出保持兼容。此次不改变 HTTP 协议、数据库迁移、序列化字段、权限规则、状态转换或数字计算规则。

此次调整模块入口及实现位置，不代表完成后端聚合状态、跨领域事务和数据所有权的解耦。

## 验证

```bash
devbox run -- make check-backend
```

`backend/tests/module_boundaries.rs` 递归扫描全部 Rust 源码模块入口，只接受文件模块声明和公开重新导出，并验证检查器能拒绝类型、函数、常量、trait、impl、普通导入及内联模块。原有 HTTP 契约、领域和序列化测试继续验证兼容行为；格式、编译和 Clippy 使用现有后端检查命令。

2026-10-28 在 Devbox / Rust 1.97.1 环境执行上述命令：格式、编译、Clippy 均通过，46 项测试通过；递归检查覆盖 24 个 `mod.rs`。需要独立 PostgreSQL URL 的 1 项集成测试按默认配置忽略，本次未执行真实数据库验证。此次仅调整 Rust 模块组织，未执行前端和真机验证。
