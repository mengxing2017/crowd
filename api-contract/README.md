# API 契约包

平台中立的 TypeScript 契约与客户端，通过调用方注入的 `Transport` 发起请求，不依赖 DOM、微信或浏览器运行时。

## 领域入口

| 入口 | 职责 | 客户端 |
| --- | --- | --- |
| `@crowd/api-contract/identity` | 登录、用户资料、收款资料、身份和参与资格 | `createIdentity` |
| `@crowd/api-contract/catalog` | 产品、分类、上架版本、目录及通知 | `createCatalog` |
| `@crowd/api-contract/attribution` | 推广码、访问触点、归因策略及申诉 | `createAttribution` |
| `@crowd/api-contract/tasks` | 佣金规则、任务模板、领取、执行、证据及审核 | `createTasks` |
| `@crowd/api-contract/governance` | 平台规则、审计及客服处理 | `createGovernance` |
| `@crowd/api-contract/common` | 传输类型、响应信封和业务错误 | — |
| `@crowd/api-contract/community` | 原有导出和通用调用的兼容入口 | `createCommunity`（不推荐新增调用） |

各领域拥有自己的 `types.ts`、`client.ts` 以及所需的响应校验、状态文案。所有 `index.ts` 只重新导出。领域之间按需使用类型依赖，不能依赖 `community`；`common` 不依赖业务领域。

```ts
import { createTasks } from '@crowd/api-contract/tasks'

const tasks = createTasks(transport)
const page = await tasks.mine(JSON.stringify({ status: 'submitted' }))
await tasks.evidenceSubmit({ id, version, preview_id, confirmed: true }, originalKey)
```

命令客户端只负责传输，业务状态、授权和数字计算仍由服务端判断。Web 和小程序分别在各 feature 的 `api/index.ts` 实例化领域客户端，共享各自的 session transport，保持原鉴权、登录持久化、错误及幂等键行为。

## 兼容边界

`community` 只组合领域客户端和旧通用读写适配器，不再拥有业务类型或校验规则。保留原有公开类型、`BusinessError`、`taskStatusNames` 和 `createCommunity` 方法，兼容已有调用。

此次是契约领域及前端 API 接入重构，不变更 HTTP 路径和后端业务行为。`/api/v1/community/*` 是现有服务端协议命名；后端聚合状态、通用命令路由及部分前端页面仍保留原结构，不能据此宣称后端已完成领域拆分。

## 验证

```bash
pnpm --dir api-contract build
pnpm --dir api-contract test
make check-web
```

契约测试在临时目录编译并加载完整模块依赖，验证入口仅包含导出、公开运行时成员、共享 Rust 序列化样本、独立领域客户端、响应校验、请求编码及幂等键；完成后清理临时产物。`make check-web` 还检查 Web 和小程序的类型、规范及构建。两端开发配置均直接解析契约源码，避免 pnpm 本地文件依赖副本遗漏新增领域文件。
