# 骨架验证

- `make check-web`：通过契约包类型检查、Web 与小程序类型检查、ESLint、React 生产构建和微信小程序生产构建。
- 后端检查：格式、Cargo check、Clippy（所有 target，拒绝 warning）通过；HTTP 测试覆盖系统 JSON 契约、存活检查和数据库不可用时的 503。
- JSON 与 YAML 配置解析通过。依赖由 Cargo.lock 和两份 pnpm-lock.yaml 固定。
- 当前环境未安装 Docker，未执行容器镜像构建、Compose 启动或真实 PostgreSQL 迁移验证。CI 配置提供源码检查；容器验证需在具备 Docker 的环境执行。
- 小程序已编译，尚未填写 AppID 或执行微信开发者工具/真机验收。

容器验证：复制 env.example 为 .env，填写密码，运行 `make compose-up`；检查 `/healthz`、`/readyz` 和 `/api/v1/system`，最后运行 `make compose-down` 保留数据卷。
