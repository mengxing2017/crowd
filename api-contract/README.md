# API 契约包

平台中立的 TypeScript 契约与客户端，通过调用方注入的 `Transport` 发起请求，不依赖 DOM、微信或浏览器运行时。

## 领域入口

| 入口 | 职责 | 客户端 |
| --- | --- | --- |
| `@crowd/api-contract/identity` | 登录、用户资料、身份、岗位／范围和会话 | `createIdentity` |
| `@crowd/api-contract/receipts` | 收款资料保存、停用及本人敏感查看 | `createReceipts` |
| `@crowd/api-contract/promotion` | 推广码归属、创建／换发、解析及二维码路径 | `createPromotion` |
| `@crowd/api-contract/catalog` | 产品、分类、上架版本、目录及通知 | `createCatalog` |
| `@crowd/api-contract/attribution` | 访问触点、归因策略及归因申诉 | `createAttribution` |
| `@crowd/api-contract/tasks` | 佣金规则、模板、参与资格、领取、执行、证据及审核 | `createTasks` |
| `@crowd/api-contract/governance` | 平台规则版本、服务协议和隐私说明 | `createGovernance` |
| `@crowd/api-contract/audit` | 审计记录及对象范围查询 | `createAudit` |
| `@crowd/api-contract/support` | 纠错、注销、合并的待核查申请 | `createSupport` |
| `@crowd/api-contract/files` | 上传／私有文件路径及上传响应校验 | 路径函数与 `decodeUpload` |
| `@crowd/api-contract/platform` | 原提交结果查询 | `createPlatform` |
| `@crowd/api-contract/common` | 传输类型、响应信封和业务错误 | — |
| `@crowd/api-contract/community` | 原有导出和通用调用的兼容入口 | `createCommunity`（不推荐新增调用） |

各领域按需拥有自己的 `types.ts`、`client.ts`、响应校验和状态文案；文件技术模块提供路径与解码器，platform 仅提供原结果查询客户端。所有 `index.ts` 只重新导出。领域之间按需使用类型依赖，不能依赖 `community`；`common` 不依赖业务领域。

```ts
import { createTasks } from '@crowd/api-contract/tasks'

const tasks = createTasks(transport)
const page = await tasks.mine(JSON.stringify({ status: 'submitted' }))
await tasks.evidenceSubmit({ id, version, preview_id, confirmed: true }, originalKey)
```

命令客户端只负责传输，业务状态、授权和数字计算仍由服务端判断。Web 和小程序分别在各 feature 的 `api/index.ts` 实例化领域客户端，共享各自的 session transport，保持原鉴权、登录持久化、错误及幂等键行为。

## 兼容边界

`community` 只组合领域客户端和旧通用读写适配器，不再拥有业务类型或校验规则。保留原有公开类型、`BusinessError`、`taskStatusNames` 和 `createCommunity` 方法，兼容已有调用。

契约职责与后端 [ADR 0009](../docs/adr/0009-community-domain-ownership.md) 对齐。HTTP 路径及后端业务行为保持兼容，`/api/v1/community/*` 是已有协议前缀。两端应用不再使用 community 门面；Web 页面位于各自业务 feature，`workspace` 仅组装控制台和导航。小程序的路由页组合领域 API 与任务组件。共享状态存储的限制仍见后端 ADR，不据此宣称独立领域仓储已全部建成。

独立客户端的原混合方法按下表迁移；旧 `createCommunity` 的方法及类型导出继续保留。

| 原调用 | 当前所属调用 |
| --- | --- |
| `identity.agreements()` | `governance.agreements()` |
| `identity.receipts/receipt/receiptDisable/receiptReveal` | `receipts` 同名方法 |
| `identity.eligibility/eligibilityVerify` | `tasks` 同名方法 |
| `attribution.codes/managedCodes/codeCreate/codeRotate/resolve` | `promotion` 同名方法 |
| `governance.audit(target)` | `audit.records(target)` |
| `governance.cases/correction/closure/merge` | `support` 同名方法 |
| 前端 `community.read('command_result', key)` | `platform.commandResult(key)` |

二进制文件传输由各端 `features/files/api/index.ts` 适配，不把 `File`、`Blob`、DOM 或 `uni` 引入契约包。文件路径和用途分别编码，证据查看仍携带原因及身份；上传响应使用共同错误类和信封校验，不将文件校验描述为官方收款验证。

前端各领域 API 导出显式命令表，`features/platform/commands.ts` 仅组合这些表、按原动作／内容／请求键调用对应客户端。未知动作不会退回通用 HTTP 命令。结果查询、待确认状态和原键重试仍由原交互流程管理。

## 验证

```bash
pnpm --dir api-contract build
pnpm --dir api-contract test
make check-web
```

契约测试在临时目录编译并加载完整模块依赖，验证入口仅包含导出、公开运行时成员、共享 Rust 序列化样本、独立领域客户端、响应校验、请求编码及幂等键；完成后清理临时产物。`make check-web` 还检查 Web 和小程序的类型、规范及构建。两端开发配置均直接解析契约源码，避免 pnpm 本地文件依赖副本遗漏新增领域文件。
