# Docker 镜像构建、发布与部署

本文参考 `../keeper/wekeeper` 的 Dockerfile 和运行手册，按 Crowd 当前配置整理。所有命令均从仓库根目录执行；版本号、仓库域名和凭据须替换为实际值。

## 交付内容与下载源

| 组件 | 构建文件 / 镜像 | 作用 |
| --- | --- | --- |
| API | `backend/Dockerfile` / `crowd-api` | Rust 1.97.1 构建，Debian bookworm slim 运行，UID/GID 10001:10001，容器端口 8080 |
| Web | `frontend/Dockerfile` / `crowd-web` | Node 22、pnpm 11.22.0 构建 contract 与 React，Caddy 提供静态资源及同源 API 代理 |
| 数据库 | `postgres:17-alpine` | 独立 PostgreSQL 17，命名卷持久化 |

`api-contract` 不单独部署；微信小程序不进入 Web 镜像，另行构建并上传微信平台。嵌入式 PostgreSQL 仅用于源码开发/测试。

下载源边界与 wekeeper 一致：

- **APT 软件包使用中科大** `mirrors.ustc.edu.cn`。API 运行阶段安装 `ca-certificates`、`curl` 前，替换传统 `sources.list` 和 Deb822 `debian.sources` 中的 Debian 主仓库、安全更新仓库地址，再更新索引、安装并清理缓存。
- 当前 API 构建阶段和 Web 镜像没有 APT/APK 安装命令。以后新增系统软件安装时，也应先配置对应的中科大软件包源。
- 基础镜像仍从官方 Docker Hub 拉取；APT 换源不等于 Docker Engine 的镜像拉取加速配置，也不将中科大软件包站点当作应用镜像发布仓库。
- Cargo 依赖仍用 crates.io；npm/pnpm 仍用 `https://registry.npmjs.org/`，与参考项目保持相同边界。这里不虚构中科大 npm 或 Cargo 镜像地址。

构建机需安装 Docker Engine、支持 `up --wait` 的 Compose v2，并能访问上述下载源。运行服务器无需 Rust、Node 或 pnpm。

```bash
docker version
docker compose version
docker buildx version
```

## 本地制作镜像

两个 Dockerfile 都要求**仓库根目录 `.` 作为构建上下文**。

```bash
make docker-build
# 或分别执行
make docker-build-api
make docker-build-web
```

等价原生命令：

```bash
docker build --pull -f backend/Dockerfile -t crowd-api:local .
docker build --pull -f frontend/Dockerfile -t crowd-web:local .
```

自定义标签通过 Make 变量传入；这两个独立构建目标不读取 `.env`：

```bash
make docker-build API_IMAGE=registry.example.com/team/crowd-api:2026.10.29-abcdef0 WEB_IMAGE=registry.example.com/team/crowd-web:2026.10.29-abcdef0
```

`--pull` 检查基础镜像更新，不固定其 digest。普通构建生成构建机原生架构镜像。跨架构发布需配置 Buildx builder，例如将构建命令改为 `docker buildx build --platform linux/amd64 --pull -f backend/Dockerfile -t "$API_IMAGE" --push .`，Web 同理；ARM 目标改为 `linux/arm64`，并在目标主机验证。

不要在构建参数中传入数据库密码、微信 AppSecret 或管理员密码。`.dockerignore` 排除 `.env`、构建产物和依赖目录，应用配置在运行时注入。

## 本地 Compose 构建与启动

首次创建配置（已有 `.env` 时直接编辑，避免覆盖）：

```bash
cp env.example .env
chmod 600 .env
openssl rand -hex 24
```

将随机结果填入 `POSTGRES_PASSWORD`，使用 URL 安全字符，因为 Compose 将其拼接到数据库 URL。镜像标签默认 `crowd-api:local`、`crowd-web:local`。首次管理员需要同时填写 `CROWD_BOOTSTRAP_ADMIN_USERNAME` 和 `CROWD_BOOTSTRAP_ADMIN_PASSWORD`（12–128 字节），成功创建后移除这两项并重建 API 容器；不会覆盖已有账号。微信配置及协议发布见 [M01–M04 手册](m01-m04.md)。

```bash
make compose-config
make compose-build
docker compose --env-file .env -f compose.yaml -f compose.dev.yaml up -d --wait
docker compose --env-file .env -f compose.yaml -f compose.dev.yaml ps
curl --fail --show-error http://127.0.0.1:8088/readyz
```

也可用 `make compose-up` 合并构建和启动。访问 `http://127.0.0.1:8088`；开发覆盖文件额外向宿主机开放 PostgreSQL 5432、API 8080 的环回端口。

```bash
docker compose --env-file .env -f compose.yaml -f compose.dev.yaml logs --tail 100 api web postgres
make compose-down
```

`down` 保留数据库卷，不要用 `down --volumes` 处理已有业务数据。开发、测试、生产使用不同目录/项目名、数据库和凭据；使用 `-p crowd-dev` 等项目名后，所有后续命令须保持相同项目名。配置检查使用 `config --quiet`，完整 `config` 会输出插值后的敏感值。

## 发布镜像与离线交付

使用实际 commit 对应的唯一版本号，避免覆盖已发布标签：

```bash
export CROWD_REGISTRY=registry.example.com/team
export CROWD_VERSION=2026.10.29-abcdef0
export API_IMAGE="$CROWD_REGISTRY/crowd-api:$CROWD_VERSION"
export WEB_IMAGE="$CROWD_REGISTRY/crowd-web:$CROWD_VERSION"
docker login registry.example.com
make docker-build
docker push "$API_IMAGE"
docker push "$WEB_IMAGE"
docker image inspect "$API_IMAGE" --format '{{json .RepoDigests}}'
docker image inspect "$WEB_IMAGE" --format '{{json .RepoDigests}}'
```

记录源码 commit、构建时间、镜像 digest 和 migration 版本。生产 `.env` 优先使用实际 digest 引用。Shell 已 export 的变量优先于 Compose `.env`，部署时清除旧值或换用干净终端。

离线交付时在联网构建机执行：

```bash
docker pull postgres:17-alpine
docker save -o crowd-images.tar "$API_IMAGE" "$WEB_IMAGE" postgres:17-alpine
sha256sum crowd-images.tar > crowd-images.tar.sha256
```

传输归档、校验文件、`compose.yaml` 和 `env.example`。目标机校验、导入，并在 `.env` 填写归档中实际标签：

```bash
sha256sum -c crowd-images.tar.sha256
docker load -i crowd-images.tar
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml up --no-build --pull never -d --wait
```

`save/load` 后不保证保留仓库 digest 元数据。镜像离线交付不代表微信登录等外部功能可离线使用。

## 生产部署与更新

生产只使用 `compose.yaml`，不叠加开发覆盖文件。准备 `.env`、发布镜像权限及独立数据库数据卷：

```bash
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml pull
docker compose --env-file .env -f compose.yaml up --no-build -d --wait
docker compose --env-file .env -f compose.yaml ps
curl --fail --show-error http://127.0.0.1:8088/healthz
curl --fail --show-error http://127.0.0.1:8088/readyz
```

Web 默认只绑定宿主机 `127.0.0.1:8088`，由同机 HTTPS 入口代理；当前 Caddy 配置仅监听容器 80，不自动提供公网 TLS。容器内入口代理不能用自身 `127.0.0.1` 访问宿主机 Web。浏览器请求经 Web 同源 `/api/*` 转发到 `api:8080`，不要把内部容器地址填给浏览器。小程序单独配置生产 HTTPS API 地址及微信合法域名。

API 启动自动执行 forward-only migration。更新前保存数据库备份、配置和旧镜像 digest；例如：

```bash
umask 077
docker compose --env-file .env -f compose.yaml exec -T postgres pg_dump -U crowd -d crowd -Fc > crowd-backup.dump
test -s crowd-backup.dump
```

备份需在隔离环境演练恢复，文件非空不等于可恢复。更新 `.env` 中镜像后重复 `pull`、`up`，检查就绪与真实业务流程。`restart` 不重新读取环境文件；修改变量后用 `up -d --force-recreate api`。PostgreSQL 初始化密码只在空数据卷生效，修改 `.env` 不会修改已有数据库角色密码。

仅当 migration 兼容时才可切回旧镜像；不兼容时前滚修复或受控恢复并核对新增记录，不能删除历史数字变更或人工登记记录回退。

## 排障与验证记录

下载失败时使用 `docker build --progress=plain ...` 查看具体阶段：基础镜像拉取检查 Docker 仓库/DNS；APT 检查中科大源连通性；Cargo/npm 检查各自公共源。不要关闭 TLS 证书校验。就绪失败查看 API migration、数据库连接和数据库日志。

2026-10-29：按本仓库及 wekeeper 配置核对构建上下文、镜像名称、端口、换源逻辑和命令。当前编辑环境未安装 Docker，未执行真实镜像构建、Compose 启动、镜像推送、离线导入或跨架构验证。这些操作须在具备 Docker 的隔离环境执行；容器就绪也不等于业务验收或官方到账验证。
