# Docker 镜像构建、发布与部署

所有命令从仓库根目录执行。先签出需要发布的代码；两个 Dockerfile 都在容器内编译，无需在宿主机提前安装 Rust、Node 或运行编译命令。构建机需要 Docker，部署需要 Compose v2（支持 `up --wait`）。

1Panel 的容器配置、HTTPS 反向代理及首次 admin 密码生成步骤见 [1Panel 部署手册](1panel.md)。

## 同时制作前后端镜像（环境变量方式）

本地默认标签一起构建：

```bash
make docker-build
# 需要禁用两端构建缓存时，改用：
make docker-build-fresh
```

发布镜像时，在同一终端配置仓库和版本，再一起构建：

```bash
export CROWD_REGISTRY=registry.example.com/team
export CROWD_VERSION=2026.10.29-abcdef0
export API_IMAGE="$CROWD_REGISTRY/crowd-api:$CROWD_VERSION"
export WEB_IMAGE="$CROWD_REGISTRY/crowd-web:$CROWD_VERSION"
make docker-build
```

将仓库地址、版本替换为实际值。Make 会读取已 export 的 `API_IMAGE`、`WEB_IMAGE`；需要完整重新构建时，将最后一条改为 `make docker-build-fresh`。

不用 Make 时，两端的等价构建命令为：

```bash
docker build --pull -f backend/Dockerfile -t "$API_IMAGE" .
docker build --pull -f frontend/Dockerfile -t "$WEB_IMAGE" .
```

构建成功后，一起推送到仓库：

```bash
docker login registry.example.com
docker push "$API_IMAGE"
docker push "$WEB_IMAGE"
```

上述变量用于镜像标签，不是数据库或管理员配置。构建命令不自动加载 `.env`；运行配置在第 3、6 节说明。已 export 的变量同样适用于下方的 `make docker-build-api`、`make docker-build-web`；如需使用它们的默认 `:local` 标签，先执行 `unset API_IMAGE WEB_IMAGE`。

## 1. 后端单独打包

构建 API 镜像：

```bash
docker build --pull -f backend/Dockerfile -t crowd-api:local .
```

或使用 Make：

```bash
make docker-build-api
```

需要完全禁用构建缓存时：

```bash
docker build --pull --no-cache -f backend/Dockerfile -t crowd-api:local .
# Make 等价命令
make docker-build-api DOCKER_BUILD_FLAGS=--no-cache
```

产物为 `crowd-api:local`，包含容器内 `cargo build --release --locked` 生成的 API 二进制；运行端口为 8080，以 UID/GID 10001:10001 运行。

## 2. 前端单独打包

构建 Web 管理端镜像：

```bash
docker build --pull -f frontend/Dockerfile -t crowd-web:local .
```

或使用 Make：

```bash
make docker-build-web
```

需要完全禁用构建缓存时：

```bash
docker build --pull --no-cache -f frontend/Dockerfile -t crowd-web:local .
# Make 等价命令
make docker-build-web DOCKER_BUILD_FLAGS=--no-cache
```

产物为 `crowd-web:local`，包含容器内构建的 React 静态资源，由 Caddy 在端口 80 提供页面并代理 API。`api-contract` 参与构建，不单独部署；微信小程序另行构建、上传微信平台。

**两端构建命令末尾的 `.` 都是仓库根目录**，不要切到 backend/frontend 子目录构建。同一代码无需重复执行 Docker 和 Make 两组等价命令。

一起打包可执行 `make docker-build`；一起禁用缓存打包可执行 `make docker-build-fresh`。构建读取当前工作目录，不自动拉取远端代码，也不读取 `.env`。普通构建允许缓存，源码内容变化会让后续编译层失效；`--pull` 仅检查基础镜像更新。

## 3. 本机启动已打包镜像

首次创建 `.env`（已有文件直接编辑，避免覆盖）：

```bash
cp env.example .env
chmod 600 .env
openssl rand -hex 24
```

将随机结果填入 `.env` 的 `POSTGRES_PASSWORD`。首次需要后台管理员时，再单独生成一份密码并填写 `CROWD_BOOTSTRAP_ADMIN_USERNAME=admin` 和 `CROWD_BOOTSTRAP_ADMIN_PASSWORD`，具体清理步骤见 [管理员初始化](1panel.md#首次安装后生成-admin-密码并初始化)。镜像配置为：

```dotenv
API_IMAGE=crowd-api:local
WEB_IMAGE=crowd-web:local
WEB_PORT=8088
```

在第 1、2 节镜像构建成功后执行：

```bash
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml up --no-build -d --wait
docker compose --env-file .env -f compose.yaml ps
curl --fail --show-error http://127.0.0.1:8088/readyz
```

访问 `http://127.0.0.1:8088`。上述编排使用独立 PostgreSQL 17，首次需能拉取 `postgres:17-alpine`；API 启动自动执行 migration。Web 绑定环回端口，数据库/API 不向宿主机开放端口。

查看日志、停止容器：

```bash
docker compose --env-file .env -f compose.yaml logs --tail 100 api web postgres
docker compose --env-file .env -f compose.yaml down
```

`down` 保留数据库卷。不要用 `down --volumes` 删除业务数据。开发、测试、生产使用不同数据库、凭据及 Compose 项目；指定 `-p` 后所有命令保持同一项目名。

需要额外开放数据库 5432、API 8080 的环回调试端口时，才叠加 `-f compose.dev.yaml`。开发快捷命令为 `make compose-config`、`make compose-build`、`make compose-up`、`make compose-down`；这些 Make 目标均包含开发覆盖文件。

## 4. 分别发布前后端镜像

将 `registry.example.com/team` 替换为实际仓库，将版本号替换为本次提交对应的唯一版本，不覆盖已发布标签。先登录仓库：

```bash
docker login registry.example.com
```

### 只发布后端

```bash
docker build --pull -f backend/Dockerfile -t registry.example.com/team/crowd-api:2026.10.29-abcdef0 .
docker push registry.example.com/team/crowd-api:2026.10.29-abcdef0
```

使用 Make 构建时对应为：

```bash
make docker-build-api API_IMAGE=registry.example.com/team/crowd-api:2026.10.29-abcdef0
```

### 只发布前端

```bash
docker build --pull -f frontend/Dockerfile -t registry.example.com/team/crowd-web:2026.10.29-abcdef0 .
docker push registry.example.com/team/crowd-web:2026.10.29-abcdef0
```

使用 Make 构建时对应为：

```bash
make docker-build-web WEB_IMAGE=registry.example.com/team/crowd-web:2026.10.29-abcdef0
```

记录源码 commit、构建时间、migration 版本及镜像 digest；推送后可查询：

```bash
docker image inspect registry.example.com/team/crowd-api:2026.10.29-abcdef0 --format '{{json .RepoDigests}}'
docker image inspect registry.example.com/team/crowd-web:2026.10.29-abcdef0 --format '{{json .RepoDigests}}'
```

普通构建针对构建机原生架构。跨架构需使用已配置好的 Buildx builder，例如后端命令：

```bash
docker buildx build --platform linux/amd64 --pull -f backend/Dockerfile -t registry.example.com/team/crowd-api:2026.10.29-abcdef0 --push .
```

前端改为 `frontend/Dockerfile` 及对应 Web 标签；ARM 目标改为 `linux/arm64`。跨架构产物须在目标主机验证。

## 5. 服务器部署与单端更新

服务器准备 `compose.yaml` 和 `.env`，填入实际发布标签，优先使用实际 digest 引用：

```dotenv
API_IMAGE=registry.example.com/team/crowd-api:2026.10.29-abcdef0
WEB_IMAGE=registry.example.com/team/crowd-web:2026.10.29-abcdef0
```

首次部署拉取并启动完整栈：

```bash
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml pull
docker compose --env-file .env -f compose.yaml up --no-build -d --wait
```

### 只更新后端

先备份数据库，将 `.env` 中 `API_IMAGE` 改为新版本，保持 `WEB_IMAGE` 不变：

```bash
docker compose --env-file .env -f compose.yaml pull api
docker compose --env-file .env -f compose.yaml up --no-build --no-deps -d --wait api
```

### 只更新前端

将 `.env` 中 `WEB_IMAGE` 改为新版本，保持 `API_IMAGE` 不变：

```bash
docker compose --env-file .env -f compose.yaml pull web
docker compose --env-file .env -f compose.yaml up --no-build --no-deps -d --wait web
```

单端更新命令适用于依赖服务已运行且健康的环境；接口不兼容的改动须配套发布。两端都更新时使用完整栈的 `pull`、`up` 命令。部署后检查：

```bash
docker compose --env-file .env -f compose.yaml ps
curl --fail --show-error http://127.0.0.1:8088/readyz
```

本地源码重新打包后，同样需要执行 `up` 替换运行容器；只有 `build` 不更新容器，`restart` 不重新编译、不拉取镜像，也不重新读取环境文件。Shell 中已 export 的同名变量优先于 `.env`，部署前清除旧值或使用干净终端。

公网 HTTPS 由同机入口代理或 1Panel 提供，当前 Caddy 只监听容器 80。入口代理若在容器内，不能用自身 `127.0.0.1` 访问宿主机 Web；网络配置见 [1Panel 手册](1panel.md)。小程序独立配置 HTTPS API 地址及微信合法域名。

## 6. 数据库与初始化配置

Compose 已将 `POSTGRES_PASSWORD` 作为独立字段传给 API，支持 `@` 等字符，无需 URL 编码。Compose `.env` 中含 `$` 的值使用单引号包裹，避免变量插值。

独立 API 容器或外部数据库连接示例：

```dotenv
CROWD_DATABASE_HOST=postgres
CROWD_DATABASE_PORT=5432
CROWD_DATABASE_NAME=crowd
CROWD_DATABASE_USER=crowd
CROWD_DATABASE_PASSWORD=实际原始密码
CROWD_DATABASE_SSL_MODE=disable
```

主机、库名、用户和密码必填；端口默认 5432，SSL 模式默认 `prefer`。示例 `disable` 用于同机容器网络；远程数据库按证书要求配置 TLS。非空 `CROWD_DATABASE_URL` 优先于分项配置，切换时移除旧 URL；保留 URL 方式时，密码特殊字符仍须百分号编码。

这些分项须使用包含新配置支持的 API 镜像。仓库 Compose 固定连接自身 PostgreSQL，修改 `.env` 中额外的分项值不会自动覆盖编排；外部数据库/1Panel 独立容器的完整配置见 [1Panel 手册](1panel.md)。

初始化管理员的两项变量仅在账号不存在时创建账号，不重置已有密码或权限；确认登录后同时清除并重建 API。微信凭据、协议发布见 [M01–M04 手册](m01-m04.md)。不要将任何运行凭据传入镜像构建参数。

## 7. 离线交付

联网构建机先完成第 1、2 节打包，再导出镜像。以下使用本地标签，发布标签需相应替换：

```bash
docker pull postgres:17-alpine
docker save -o crowd-images.tar crowd-api:local crowd-web:local postgres:17-alpine
sha256sum crowd-images.tar > crowd-images.tar.sha256
```

传输归档、校验文件、`compose.yaml`、`env.example`。目标机导入，在 `.env` 填写归档中的实际标签：

```bash
sha256sum -c crowd-images.tar.sha256
docker load -i crowd-images.tar
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml up --no-build --pull never -d --wait
```

离线更新同样须重新导出、导入新镜像并更新运行配置。`save/load` 不保证保留仓库 digest 元数据。镜像离线交付不代表微信登录等外部功能可离线使用。

## 8. 下载源、备份与回退

| 下载内容 | 来源 |
| --- | --- |
| API 运行阶段 APT 软件包及安全更新 | 中科大 `mirrors.ustc.edu.cn` |
| Rust、Node、Debian、Caddy、PostgreSQL 基础镜像 | 官方 Docker Hub |
| Rust 依赖 | crates.io |
| npm/pnpm 包 | `https://registry.npmjs.org/` |

当前构建阶段没有 APT/APK 安装命令，后续新增系统软件安装时先配置对应中科大源。APT 换源不改变 Docker 镜像拉取源。构建上下文排除 `.env`、`target`、`dist`、`node_modules`、`*.tsbuildinfo`，不打包宿主机编译结果。

API 启动自动执行 forward-only migration。更新前备份数据库、配置和旧镜像 digest：

```bash
umask 077
docker compose --env-file .env -f compose.yaml exec -T postgres pg_dump -U crowd -d crowd -Fc > crowd-backup.dump
test -s crowd-backup.dump
```

备份须在隔离库演练恢复，非空不等于可恢复。PostgreSQL 初始化变量只在空卷生效，改 `.env` 不会修改已有数据库角色密码。

只有 migration 兼容时才能切回旧镜像；不兼容时前滚修复或受控恢复并核对新增记录，不能删除历史数字变更或人工登记记录回退。

## 排障与验证记录

下载失败时使用 `docker build --progress=plain ...` 查看具体阶段：基础镜像拉取检查 Docker 仓库/DNS；APT 检查中科大源连通性；Cargo/npm 检查各自公共源。不要关闭 TLS 证书校验。就绪失败查看 API migration、数据库连接和数据库日志。

2026-10-29：按本仓库配置核对构建上下文、镜像名称、端口、换源逻辑和命令。当前编辑环境未安装 Docker，未执行真实镜像构建、Compose 启动、镜像推送、离线导入或跨架构验证。这些操作须在具备 Docker 的隔离环境执行；容器就绪也不等于业务验收或官方到账验证。

本次构建流程复查：已核对源码复制路径及 Make 强制构建命令，排除了本地 TypeScript 增量状态文件。当前环境仍无 Docker，尚未实测“修改源码 → 重建镜像 → 替换容器”的运行回归。

### 数据库分项连接配置验证（2026-10-29）

API 新增 `CROWD_DATABASE_HOST`、`CROWD_DATABASE_PORT`、`CROWD_DATABASE_NAME`、`CROWD_DATABASE_USER`、`CROWD_DATABASE_PASSWORD`、`CROWD_DATABASE_SSL_MODE`，通过 SQLx 连接选项直接传入原始密码。非空 `CROWD_DATABASE_URL` 保持优先；使用分项时清除旧 URL。Compose 已改为分项传参，须配套部署包含此修改的新 API 镜像，旧二进制仍只接受 URL。

Devbox / Rust 1.97.1 下格式、`cargo check --locked` 和 Clippy（all-targets、拒绝警告）通过。`cargo test --locked --bin crowd-api` 的 3 项测试通过，覆盖特殊字符密码保留、URL 优先与空值回退、配置校验及错误脱敏。完整 `make check-backend` 在测试产物生成阶段因磁盘空间耗尽中断，未完成全量测试；清理本包可再生构建产物后完成上述专项测试。未执行真实数据库密码认证、Compose 启动或 1Panel 部署验证。
