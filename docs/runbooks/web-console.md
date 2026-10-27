# Web 管理端重构与验证

## 参考与范围

参考 `/home/dev/test/ant-design-pro` 的 `config/defaultSettings.ts`、`src/pages/Welcome.tsx` 与 `src/pages/user/login/index.tsx`，以及 [Ant Design Pro 在线预览](https://preview.pro.ant.design/welcome/)、[ProLayout 文档](https://procomponents.ant.design/components/layout/)、[PageContainer 文档](https://procomponents.ant.design/components/page-container/)。采用与本地参考一致的 Ant Design 6.6.7、Pro Components 3.1.14-2、Icons 6.3.4，保留 Vite、React 与原有 API 契约，不引入 Umi 服务或参考项目的模拟业务数据。

本次仅重构 `frontend/apps/web`，小程序仍独立构建。没有新增任务、佣金、申请或人工付款接口，也没有支付对接。业务规则依据仍为 M01–M04 需求追踪记录；布局调整不代表业务规则变更。

## 页面与交互

| 页面 | 当前能力 | 服务端岗位 |
| --- | --- | --- |
| 登录 | LoginForm、账号密码/一次性票据切换、字段校验、提交状态与错误追踪 | 登录后校验有效后台岗位 |
| 工作台 `#/welcome` | 当前岗位、范围、真实规则版本统计、快捷入口、业务路径 | 全部已授权工作人员 |
| 业务规则 `#/rules` | 类型/状态筛选、版本表格、详情、草稿抽屉、发布/停用确认 | rule_manager |
| 用户与权限 `#/users` | 按用户编号查询、状态、岗位与范围表单、业务限制 | identity_reviewer |
| 推广码 `#/promotions` | 归属查询、有效/历史码、原因与审批依据表单、撤销换发 | promotion_manager |
| 审计 `#/audit` | 按业务对象查询最近 100 条、结果、请求编号与原记录详情 | auditor |
| 使用指南 `#/guide` | 首次使用流程、岗位职责、数字与线下处理边界、常见问题 | 全部已授权工作人员 |
| 公开推广入口 `?source=...` | 实际解析结果、加载/无效/复制反馈 | 无需登录 |

ProLayout 提供固定头部、响应式侧栏、账号菜单和面包屑；ConfigProvider 统一中文与蓝色主题。导航按岗位显示，直接输入未授权 hash 回到工作台；服务端继续逐请求授权。规则统计来自本账号实际授权的规则响应，不显示虚构的平台用户数、资金池或结算数字。

服务协议与隐私说明使用正文表单；其他既有规则仍保留结构化配置，不能静默丢弃未定义字段。用户权限和换码改为独立字段表单，不要求经办拼装请求对象。用户列表接口尚未提供，当前仅支持完整编号精确查询，不以客户端模拟全量用户搜索。

提交沿用现有唯一请求键、版本与原因。未知结果在页面保留原动作、内容、所属账号和请求键，允许查询原结果或原请求重试；未知结果未解决前阻止新写入。确定成功后刷新失败只提示读取错误，不再把已确认提交当作未确认或要求重复写入。退出清理会话与待确认状态；会话到期重新登录时，不向另一账号复用待确认请求。

会话令牌保存在当前标签页的 `sessionStorage`，刷新后先向 `/read/me` 校验有效状态与岗位，再恢复页面和规则；不缓存用户权限作为授权依据。401、无效岗位和主动退出清除令牌；临时网络或服务故障保留令牌并提供“重试恢复”和“返回登录”。浏览器禁用存储时退回内存会话，关闭标签页结束该标签页的保存会话。开发管理员控制台凭据输出方式见 [嵌入式 PostgreSQL 运行手册](embedded-postgres.md)。

## 开发与检查

```bash
# 启动 API 与 Web（不同终端）
devbox run -- make api-dev
make web-dev
# 契约、两端类型/lint/构建
make check-web
# 首次安装 Chromium；Linux CI 可增加 --with-deps
pnpm --dir frontend --filter @crowd/web exec playwright install chromium
# 独立 5178 端口，不复用或停止正在使用的 5173 开发服务
pnpm --dir frontend --filter @crowd/web test:e2e
# 验证已经构建的生产静态资源
CROWD_WEB_E2E_PREVIEW=1 pnpm --dir frontend --filter @crowd/web test:e2e
```

浏览器测试位于 `frontend/apps/web/tests/console.spec.ts`。使用 Chromium 和显式拦截的隔离接口夹具，验证 UI 发送的版本、范围和幂等键，不连接真实用户库，不代表真实管理员认证、真实数据库或人工结算验收。覆盖账号登录、规则保存/发布、受限菜单与直接 hash、刷新恢复会话与退出清理、换码超时原请求重试、用户权限表单、已确认写入后的刷新失败、390px 移动端无横向溢出。测试截图和失败 trace 保存在被 Git 忽略的 `frontend/apps/web/test-results/`。

## 执行证据

2026-10-27，在 Linux x86_64、pnpm 11.22.0、Rust 1.97.1 与 Playwright Chromium 156 上实际执行：

| 检查 | 结果 |
| --- | --- |
| `CI=true make check-web` | 通过：契约校验、两端类型/lint、Web 与小程序生产构建 |
| `devbox run -- make check-backend` | 通过：fmt、编译、Clippy、11 项领域及 3 项 HTTP 契约测试；默认忽略的数据库测试本次未重跑 |
| 开发服务浏览器回归 | 4 项核心流程通过：登录与规则、角色与 hash、未知结果重试、移动端 |
| `CROWD_WEB_E2E_PREVIEW=1 ... test:e2e` | 最终生产静态资源上 6 项全部通过，增加用户权限请求及已确认写入后刷新失败恢复 |
| 截图检查 | 已生成并检查登录/工作台桌面截图及 390px 移动截图；移动端没有页面横向溢出 |
| 文档与差异 | 本次本地文档引用和 Git whitespace 检查通过 |

业务页面按需加载，React、Pro 与 Ant Design 资源分包。当前 Ant Design 公共 chunk 约 950 kB（gzip 305 kB），Vite 仍给出大于 500 kB 的体积提示；未提高告警阈值或宣称完成首屏性能指标验收。

浏览器测试使用隔离接口夹具，未在本轮重复真实 PostgreSQL、微信或人工结算联调；既有数据库证据见 [M01–M04 运行记录](m01-m04.md)。未执行人工业务验收、生产镜像发布或远端 GitHub Actions。CI 已配置在构建后安装 Chromium 并运行生产静态资源回归。


## 刷新保持登录修复（2026-10-27）

修复原先刷新丢失内存令牌的问题：仅 Web API 适配器保存令牌，不保存密码、完整用户资料或权限快照。启动恢复显示加载状态，不先闪现登录表单；恢复成功保留当前 hash 页面，受限角色仍无法进入未授权页面。过期令牌会清理，临时错误可重试。旧请求的 401 只能清理对应令牌，不覆盖较新登录。小程序登录方式不受影响；未确认业务操作引用仍仅在内存中，刷新后应通过原结果/审计辅助核对。

新增浏览器回归覆盖刷新保留页面与岗位边界、退出后刷新不恢复、401 清理及重新登录、503 保留会话并可重试恢复。本次实际结果：开发服务与生产静态资源各 9 项 Chromium 测试全部通过；契约、两端类型/lint 与两端构建检查通过。初次完整检查受宿主机文件监听额度影响出现 EMFILE 日志，使用 `CHOKIDAR_USEPOLLING=1 CI=true make check-web` 重跑后通过且无该错误，记录位于 `/tmp/crowd-refresh-check-web.log`。本轮仅改 Web 会话恢复与说明，未重跑真实数据库或微信联调，不把接口夹具测试称为真实身份验证。
