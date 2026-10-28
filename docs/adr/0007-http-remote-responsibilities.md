# ADR 0007：HTTP remote 按实体和用例拆分

## 问题与决策

原 `community/north/remote.rs` 同时实现登录、产品目录、推广码、文件、任意查询及任意命令。只按函数排列不能明确接口归属，新业务容易继续追加到通用入口。

将其替换为 remote 目录，各文件承担一个实体或用例的 HTTP 适配；`mod.rs` 只声明和导出，`routes.rs` 只组装路由。已有实体查询和命令各自拥有具名 handler，handler 绑定固定操作名，不从请求体选择其他操作。

## 目录职责

| 位置 | 职责 |
| --- | --- |
| `community/north/remote` | 独立的用户登录、管理员登录、会话兑换、协议、用户资料、收款资料、推广码、规则、审计及客服接口 |
| `catalog/north/remote` | 公开目录、公开产品图片、访问触点、产品、分类及归因接口 |
| `tasks/north/remote` | 定价、模板、规则查询、任务实例、证据、审核、关联更正及质量检查接口 |
| `system/north/remote` | 已按健康检查和系统信息拆分，本次保留 |
| `http` | 共用追踪编号、Bearer token 提取、JSON 解析、响应信封及调用既有用例的协议适配 |

`http` 不包含实体路由表、SQL、业务权限判定、状态转换或数字规则。认证、资源归属、数据范围、状态及幂等仍由原服务端用例检查。

## 协议兼容

- 保留所有 HTTP 路径、方法、响应结构、状态映射和请求追踪编号。
- 已知 `/api/v1/community/read/<section>` 及 `/api/v1/community/commands/<action>` 以完整路径注册到实体 handler。URL 命名兼容不代表 handler 仍集中在 community。
- `compatibility.rs` 保留原动态路径适配，不承载新增业务。具体路由优先于通配路由，不更改原有未知操作的处理方式。
- 保留原 `community::north::remote::routes()` 工厂作为路由组合入口，内部合并目录与任务各自的 remote 路由；不再在其中实现实体 handler。
- 保留原 JSON 和文件请求限制、Idempotency-Key 校验顺序、`no-store` 和 `nosniff` 响应头、文件用途及归属授权。原文件 URL 的授权用途参数继续兼容。
- 不新增业务规则、数据库迁移和对外接口；契约包无需改变。

## 验证

```bash
devbox run -- make check-backend
devbox run -- make test-business
```

HTTP 契约测试覆盖拆分后各类命令的缺少幂等键、非法 JSON、错误信封、文件大小上限和用途参数错误，以及旧动态路径兼容。独立嵌入式 PostgreSQL 集成测试对比各实体查询与原用例结果，验证鉴权、权限拒绝、原键重复提交、原结果恢复、资料版本不重复变化及私有图片越权拒绝。原领域测试继续覆盖任务和数字一致性。

此次仅拆分 HTTP 适配层；后端聚合状态和跨领域事务仍使用原实现，不据此宣称领域解耦或业务验收完成。

2026-10-28 在 Devbox / Rust 1.97.1 环境完成验证：`check-backend` 的格式、编译、Clippy 和 48 项测试均通过，模块入口检查覆盖 28 个 `mod.rs`；默认忽略的 PostgreSQL 集成测试另通过 `test-business` 在独立临时 PostgreSQL 17 中执行，1 项通过。未执行前端或小程序真机验证。
