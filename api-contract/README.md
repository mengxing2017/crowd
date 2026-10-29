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
| `@crowd/api-contract/support` | 资料核查申请、客服工单、申诉及服务统计 | `createSupport` |
| `@crowd/api-contract/campaigns` | 活动计划、可选额度及统计 | `createCampaigns` |
| `@crowd/api-contract/notifications` | 消息、模板、授权与岗位待办 | `createNotifications` |
| `@crowd/api-contract/risk` | 风险规则、核查与具体限制 | `createRisk` |
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

## M13–M16 领域入口

新增 `@crowd/api-contract/commissions`、`/withdrawals`、`/settlements`，分别负责数字记录与核对、规则与申请、人工经办与登记复核。金额为 CNY 安全整数分；`withdrawalStatusNames` 明确区分待人工处理、待核实、付款登记待复核与已登记结算。`commission-summary.json` 由 Rust 与 TypeScript 共用验证。

每笔确认使用同一请求键；最终申请绑定预览、汇总、原收款资料及规则版本。超额／版本冲突的错误 `details` 可携带最新汇总与本人申请，不能静默缩额。原结果通过 platform 查询；不得把重试变为再次线下付款。完整调用及权限见 [M13–M16 手册](../docs/runbooks/m13-m16.md)。契约不提供支付、查单或自动付款功能。

## 收款资料与人工异常（M17–M19）

`receipts.receiptCleanup` 仅清理本人已停用、超过显式保留期且无历史申请或核查保全引用的微信码图；请求携带资料编号、当前用户版本和用途，原请求键用于恢复。资料新增本人声明版本／确认时间，列表推导默认版本；支付宝无明确格式政策时仍是草稿，禁止新申请使用。

`settlements.settlementException` 接收申请编号／版本、异常类型、核查责任人及说明。异常类型为 `receipt_error`、`unpaid`、`unknown`、`registration_error`，保留数字占用；责任人不会自动替换独占经办。异常后登记需 `offline_result_confirmed`，可选 `reference_note` 仅记必要线下流水备注。资料错误不能继续登记，已登记错误走关联更正。

配置、兼容边界和未验证项见 [M17–M19 运行手册](../docs/runbooks/m17-m19.md)。不引入支付、自动查单或官方账户验证。

## M20–M24 新增接口

所有方法通过已有 `Transport`、统一响应信封、身份校验和原请求键工作。固定查询路径为 `/api/v1/community/read/{section}`，固定命令路径为 `/api/v1/community/commands/{action}`。兼容 URL 名称不改变领域归属。

| 所属包 | 查询方法／section | 命令方法／action |
| --- | --- | --- |
| commissions | `adjustments(target?)`／`adjustments` | `adjustmentPropose`、`adjustmentLocate`、`adjustmentReview`、`adjustmentBatchPreview`、`adjustmentBatchReview`；对应同名 snake_case action |
| campaigns | `campaigns()`、`campaignReport(id)` | `campaignSave`、`campaignTransfer`、`campaignArchive` |
| notifications | `messages()`、`notificationTemplates()`、`notificationTodos()` | `notificationRead`、`notificationConsent`、`notificationTemplate` |
| support | `tickets()`、`supportPolicy()`、`supportStatistics()` | `ticketCreate`、`ticketAssign`、`ticketReply`、`ticketClose`、`ticketReopen`；`supportPolicySave` 对应 `support_policy` |
| risk | `riskRules()`、`riskCases()`、`myRisk()` | `riskRulePublish`、`riskScan`、`riskCaseOpen`、`riskCaseDecide` |

修改已有对象使用 `id` 与最新 `version`；首次创建的活动、模板、风险规则或工单政策版本为 0。所有命令保留原请求键；同键不同内容返回 `IDEMPOTENCY_CONFLICT`。批次保存条目编号与版本，独立复核重新验证。

调整依据包含 `owner`、`task_id`、`basis_key`、`source`、`evidence`、`amount_cents`、`reason`；部分失效可传 `invalid_base_cents`，恢复传 `restores`。`amount_cents` 为正整数分，方向由是否恢复确定；待审取消不产生确认数调减。`pending_location`、`pending`、`awaiting_resolution`、`applied`、`rejected` 分别表示待定位、待独立复核、关联业务待核查、已生效、已驳回；`dispute_recorded` 表示已登记结算争议经明确人工结论记录结案，不产生自动追偿或未来抵扣。

工单创建包含 `category`、`description`、`related_id`、可选 `evidence`；回复包含 `body`、`internal`、`attachments` 与原因。附件使用 `support_attachment` 上传用途；查看采用现有 `filePath(id, reason)`，服务端重新校验工单与内部备注可见性。

风险规则条件显式配置；自动命中只生成待核查单。`riskCaseDecide` 接受 `observe`、`restrict`、`hold_source`、`release`。`restrict` 限定 `scope` 为 `task_claim` 或 `withdrawal_apply` 并要求 `expires_at`；`hold_source` 指定 `source`、`amount_cents`，还需数字审核权限。数字暂缓解除需要复核权限。

外部发送不通过浏览器确认：`notificationDelivery` 始终拒绝伪造渠道结果。可信通知适配器使用 Rust `NotificationDelivery`／`DeliverNotification`；发送必须有用户模板授权及显式有限重试策略。站内通知不依赖外部渠道成功。
