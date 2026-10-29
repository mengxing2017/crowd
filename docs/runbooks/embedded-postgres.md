# 嵌入式 PostgreSQL 开发与验证

本项目使用 `postgresql_embedded 0.21` 管理真实 PostgreSQL 17 子进程，业务仍通过 SQLx/PostgreSQL 适配器访问数据库，不是内存数据库或数据库模拟。

## 运行

```bash
# 本地 API：无需 CROWD_DATABASE_URL、Docker 或手动建库
devbox run -- make api-dev
# 相同入口
devbox run -- make dev-embedded
# 另一个终端启动 Web
devbox run -- make web-dev
```

首次运行可能联网下载服务端二进制，后续复用 `~/.theseus/postgresql` 安装缓存。数据库仅监听 127.0.0.1，使用随机端口、随机密码和临时数据目录，自动创建 `crowd_dev`。API 默认 `127.0.0.1:8080`，可用 `CROWD_BIND_ADDR=127.0.0.1:其他端口` 修改；嵌入式入口拒绝非环回 API 地址。

`CROWD_DATABASE_URL` 不参与此入口配置，`CROWD_TEST_DATABASE_URL` 也不会用于开发库。不会自动读取 `.env`。同终端 export 的微信 AppID/AppSecret、推广域名等继续生效；生产 `CROWD_BOOTSTRAP_ADMIN_*` 凭据不参与嵌入式入口初始化；不会添加假微信身份或自动绕过注册协议。

正式 API 与开发入口共用 `backend/src/runtime.rs`：执行同一组 migration、启动推广码重试 worker，使用同一 Router 与优雅关闭流程。关闭顺序为 HTTP 排空、停止 worker、关闭 SQLx 连接池、停止 PostgreSQL、释放临时数据和密码文件；数据库创建、迁移或 HTTP 启动失败也走清理路径。日志输出数据库端口和数据目录用于诊断，不输出连接 URL 或数据库密码。

Ctrl+C/SIGTERM 正常退出删除临时库，**重启后用户、规则、上传图片及申请等开发数据均重置**。需要保留数据时使用 `devbox run -- make api-external` 并显式设置外部数据库 URL。SIGKILL/系统崩溃不保证清理，不将意外残留目录用作正式数据。生产 Docker 只构建正式二进制，继续连接独立 PostgreSQL，不打包嵌入式开发依赖。

## 自动验证

```bash
devbox run -- make check-backend
devbox run -- make test-embedded
devbox run -- make test-business
```

`test-embedded` 先编译实际 example，再用 Python 3 标准库驱动真实进程，验证：

- `/readyz` 和 `/api/v1/system` 正常，证明真实迁移与服务启动；继承故意无效的外部数据库地址仍正常运行。
- SIGINT 和 SIGTERM 各自退出成功，数据库端口不再监听，临时目录已删除。
- HTTP 端口占用时返回非零，已启动的数据库及临时目录仍清理。
- 非环回 HTTP 地址在数据库启动前拒绝；外部数据库凭据不出现在日志。

此信号检查面向 POSIX；Windows 需另验 Ctrl+C 行为。

`test-business` 使用同一嵌入式管理器创建全新 `crowd_test_embedded` 数据库，覆盖继承的外部测试库 URL，执行 `community_postgres` 中显式忽略的真实数据库测试；同时配置测试专用推广 URL，覆盖 HTTP 图片与二维码输出。子进程失败或测试中断均返回失败，并停止临时数据库。每次重新建库，不回退连接应用/生产数据库。测试只验证代码和数据库行为，不代表真实微信、真机或业务验收通过。

## 本次执行记录

2026-10-27，Linux x86_64、Rust 1.97.1，复用 Theseus PostgreSQL 17.11.0 安装缓存：

| 命令/检查 | 本地实际结果 |
| --- | --- |
| `devbox run -- make test-embedded` | 通过：迁移/就绪、SIGINT、SIGTERM、端口占用失败清理、非环回拒绝、外部配置隔离 |
| `devbox run -- make test-business` | 通过：全新临时 PostgreSQL 上的并发、幂等、文件权限、资料版本、会话与 HTTP/PNG 集成测试 |
| `devbox run -- make check-backend` | 通过：格式、编译、Clippy 全 target、11 个领域测试、3 个 HTTP 契约测试；默认忽略的数据库测试已通过上一命令单独执行 |
| 正式依赖图 | `cargo tree --locked --edges normal` 不含 postgresql_embedded/archive/commands |
| 配置和文档 | Python 语法、Devbox JSON、本次文档本地引用、Git whitespace 检查通过 |

测试开发实例均已正常停止，临时目录删除；安装缓存保留。生命周期和业务测试已加入 CI 配置，但本次未执行远端 GitHub Actions。未执行生产镜像构建、真实微信联调或真机验收；不把数据库入口验证描述为业务验收。

## 管理员账号密码登录

启动 `devbox run -- make api-dev` 后，Web 首页即管理员登录页。嵌入式数据库自动创建账号 `admin`，日志事件 `development administrator credentials ready` 的 `credential_file` 为本次临时凭据文件。开发后端控制台直接输出默认用户名 `admin` 和本次随机生成的临时密码，也可用本机 `cat <credential_file 路径>` 读取。文件权限为 `0600`，退出随临时 PG 数据一并删除。按用户要求仅本地开发入口显示临时凭据，不写入结构化业务日志；正式 API 不输出管理员密码。可在启动前设置 `CROWD_DEV_ADMIN_PASSWORD`（12–128 字节），不设置则每次随机生成；设置后控制台提示使用该变量的值，不重复打印自定义密码。

此前“无假微信身份”的边界继续成立：管理员账号是独立工作人员登录渠道，不是开发微信身份后门。无需微信配置即可进入后台发布协议；小程序用户注册仍需真实微信凭证。Web 令牌保存在当前标签页 sessionStorage，刷新后向服务端重新校验并恢复；过期或退出后需重新登录。

新增验证覆盖：真实 HTTP 密码登录并查询授权规则；凭据文件权限、开发控制台默认凭据展示及结构化日志不含密码；Argon2id 哈希、错误密码、未知账号、连续失败阻断与到期恢复、禁用拒绝、退出失效，以及重复 bootstrap 不覆盖密码。2026-10-27 本次实际执行结果：

| 命令 | 结果 |
| --- | --- |
| `devbox run -- make check-backend` | 通过：fmt、编译、Clippy（零警告）、11 个领域测试及 3 个 HTTP 契约测试；数据库测试单独执行如下 |
| `CI=true make check-web` | 通过：管理员契约请求与响应校验、两端类型检查/lint/生产构建 |
| `devbox run -- make test-embedded` | 通过：实际 HTTP 管理员登录与授权规则读取、凭据文件 `0600`、开发默认凭据输出/自定义密码提示、结构化日志无密码、生产 bootstrap 配置隔离、信号退出与目录清理 |
| `devbox run -- make test-business` | 通过：真实 PostgreSQL 上密码哈希、登录正反向、失败阻断恢复、禁用、退出、bootstrap 不覆盖及原 M01–M04 回归 |

沿用本页 Linux / Rust / PostgreSQL 运行环境。未执行人工浏览器交互验收、真实微信联调、真机或生产发布；不将构建和自动 HTTP 测试等同业务验收。
