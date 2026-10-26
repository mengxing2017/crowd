# 嵌入式 PostgreSQL 开发与验证

本次参考 `../wekeeper/backend/examples/embedded_postgres.rs` 和生命周期检查方式，使用 `postgresql_embedded 0.21` 管理真实 PostgreSQL 17 子进程，业务仍通过 SQLx/PostgreSQL 适配器访问数据库，不是内存数据库或数据库模拟。

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

`CROWD_DATABASE_URL` 不参与此入口配置，`CROWD_TEST_DATABASE_URL` 也不会用于开发库。不会自动读取 `.env`。其他同终端 export 的微信 AppID/AppSecret、管理员初始化身份、推广域名等继续生效；不会添加假微信身份或自动绕过注册协议。

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
