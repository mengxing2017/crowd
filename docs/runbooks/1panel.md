# 1Panel 部署与首次管理员初始化

本文使用已发布的 Crowd API/Web 镜像，部署过程无需安装 Rust、Node 或重新编译。源码打包、镜像发布与离线导入见 [Docker 手册](docker.md)。示例域名、版本和密码均须替换。1Panel 不同版本的菜单名称可能不同，以下按编排、容器、网络和网站反向代理所需配置说明。

## 账号与配置的区别

| 项目 | 来源与用途 |
| --- | --- |
| PostgreSQL 用户 `crowd` | 数据库连接账号，不是后台登录账号 |
| `POSTGRES_PASSWORD` | 部署者生成的数据库密码，没有默认值 |
| 后台用户 `admin` | 通过 `CROWD_BOOTSTRAP_ADMIN_USERNAME=admin` 显式创建 |
| 后台密码 | 部署者生成或设置的 `CROWD_BOOTSTRAP_ADMIN_PASSWORD`，12–128 字节 |
| 1Panel 登录账号 | 由 1Panel 自身管理，与上述账号独立 |

生产没有默认 admin 密码，也不会打印自动生成的后台密码。嵌入式开发环境的随机临时管理员不适用于部署数据库。初始化变量只创建不存在的账号，不覆盖已有账号的密码或岗位权限；更换变量不能重置已有 `admin` 密码。

## 方案一：导入 Compose 编排

推荐直接使用仓库的 `compose.yaml`。不要同时创建方案二的同名独立容器。

1. 在服务器准备专用目录，例如 `/opt/crowd`，放入 `compose.yaml` 和 `env.example`。在 1Panel 编排中导入该文件，并配置工作目录/环境文件为该目录；生产不导入 `compose.dev.yaml`。
2. 如使用私有镜像仓库，在 1Panel 配置对应仓库凭据，或在用于部署的 Docker 环境执行 `docker login`。离线交付则先按 Docker 手册执行 `docker load`。
3. 首次创建 `.env`，填入实际值；已有文件直接编辑，不要覆盖。

```bash
cd /opt/crowd
cp env.example .env
chmod 600 .env
# 两次分别生成数据库密码和管理员密码，安全保存各自用途。
openssl rand -hex 24
openssl rand -hex 24
```

`.env` 完整配置示例：

```dotenv
POSTGRES_PASSWORD=替换为第一次生成的48位十六进制字符串
API_IMAGE=registry.example.com/team/crowd-api:实际版本
WEB_IMAGE=registry.example.com/team/crowd-web:实际版本
WEB_PORT=8088
CROWD_WEB_LISTEN=:80
CROWD_API_UPSTREAM=api:8080
CROWD_BOOTSTRAP_ADMIN_USERNAME=admin
CROWD_BOOTSTRAP_ADMIN_PASSWORD=替换为第二次生成的48位十六进制字符串
CROWD_BOOTSTRAP_PLATFORM_ID=
CROWD_SESSION_SECONDS=7200
CROWD_PROMOTION_ORIGIN=https://crowd.example.com
CROWD_WECHAT_APP_ID=
CROWD_WECHAT_APP_SECRET=
```

微信 AppID/AppSecret 使用实际服务端凭据，未配置时不具备可信微信登录能力。管理员账号密码登录无需将微信身份配置为管理员。`CROWD_PROMOTION_ORIGIN` 用于普通推广二维码的 HTTPS 入口，不是 Caddy 监听地址或浏览器鉴权 Origin 配置。

4. 在 1Panel 中拉取镜像、启动编排；也可在服务器终端执行：

```bash
cd /opt/crowd
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml pull
docker compose --env-file .env -f compose.yaml up --no-build -d --wait
docker compose --env-file .env -f compose.yaml ps
curl --fail --show-error http://127.0.0.1:8088/readyz
```

离线部署跳过 `pull`，启动增加 `--pull never`。1Panel 和命令行必须使用同一工作目录、项目名及环境配置，避免建立第二套数据库卷；若面板设置了项目名，所有终端 Compose 命令增加相同 `-p 项目名`。

Compose 固定数据库名/用户为 `crowd`，API 监听 `0.0.0.0:8080`、数据库主机 `postgres:5432`。`.env` 只有被 `compose.yaml` 引用的变量才会传入容器；仅添加 `CROWD_DATABASE_URL` 不会覆盖编排中的分项连接配置。使用已有外部数据库时选方案二，或明确修改编排中的 API 连接和数据库依赖。

## 方案二：1Panel 分别创建三个容器

三个容器加入同一自定义网络 `crowd-standalone`。以下名称和网络别名要保持一致，尤其是 **API 的网络别名必须包含 `api`**，因为 Web 默认上游为 `api:8080`；自定义地址见下方 Web 环境变量。

| 容器名 | 镜像 | 网络别名 | 端口映射 | 持久化 |
| --- | --- | --- | --- | --- |
| `crowd-postgres` | `postgres:17-alpine` | `postgres` | 无需对公网映射 | 命名卷 `crowd-standalone-postgres` → `/var/lib/postgresql/data` |
| `crowd-api` | 实际 API 发布镜像 | `api` | 无需对公网映射 | 业务数据写入 PostgreSQL |
| `crowd-web` | 实际 Web 发布镜像 | `web` | `127.0.0.1:8088` → 容器 `80` | 可保留 Caddy `/data`、`/config` 命名卷 |

重启策略均设 `unless-stopped`。Web 使用容器端口 80；API 使用 8080，不能在 API 内监听 `127.0.0.1`。端口映射若面板支持绑定地址，填写 `127.0.0.1`。

### 数据库环境变量

可保存为 `.env.postgres`，或逐项填写到 1Panel 数据库容器环境配置：

```dotenv
POSTGRES_DB=crowd
POSTGRES_USER=crowd
POSTGRES_PASSWORD=替换为生成的数据库密码
```

首次空卷启动创建数据库及角色。更改环境变量不会修改已有数据卷中的数据库密码。使用已有外部数据库时，不创建此容器，向数据库维护者获取专用数据库、应用角色和 TLS 要求。

### API 环境变量

可保存为 `.env.api`：

```dotenv
CROWD_BIND_ADDR=0.0.0.0:8080
CROWD_DATABASE_HOST=postgres
CROWD_DATABASE_PORT=5432
CROWD_DATABASE_NAME=crowd
CROWD_DATABASE_USER=crowd
CROWD_DATABASE_PASSWORD=替换为同一数据库原始密码
CROWD_DATABASE_SSL_MODE=disable
CROWD_BOOTSTRAP_ADMIN_USERNAME=admin
CROWD_BOOTSTRAP_ADMIN_PASSWORD=替换为另行生成的管理员密码
CROWD_SESSION_SECONDS=7200
CROWD_PROMOTION_ORIGIN=https://crowd.example.com
CROWD_WECHAT_APP_ID=
CROWD_WECHAT_APP_SECRET=
RUST_LOG=info
```

分项密码原样填写，支持 `@`、`:`、`/`、`%` 等字符，无需 URL 编码。外部数据库将主机、库名、角色和密码替换为实际值；远程连接按证书要求设置 `CROWD_DATABASE_SSL_MODE=verify-full` 并配置受信任 CA；角色须有执行 migration 所需权限。容器内 `localhost` 是 API 容器自身，不是数据库或宿主机。

`docker run --env-file` 不展开 `$变量`，填写实际值，不要写 `$POSTGRES_PASSWORD` 占位。API 启动先迁移，再执行管理员初始化，最后监听 HTTP。

### Web 环境变量

可保存为 `.env.web`，或逐项填写到 1Panel 的 Web 容器环境配置：

```dotenv
CROWD_WEB_LISTEN=:80
CROWD_API_UPSTREAM=api:8080
```

| 变量 | 默认值 | 用途 |
| --- | --- | --- |
| `CROWD_WEB_LISTEN` | `:80` | Caddy 容器内监听地址；1Panel 端口映射和入口代理须与其端口一致 |
| `CROWD_API_UPSTREAM` | `api:8080` | Web 容器可访问的 API 主机名及端口；不带 `/api` 路径 |

默认上游对应前述 API 网络别名 `api`。如果 API 容器名为 `crowd-api` 且在同一网络，也可填写 `CROWD_API_UPSTREAM=crowd-api:8080`；外部 API 则填写实际可达的上游地址。容器内 `127.0.0.1` 是 Web 容器自身，不能用它访问另一个 API 容器。

这两个变量在容器启动时由 Caddy 读取，无需为不同 API 地址重新编译 React。首次启用需要使用包含环境变量支持的新 Web 镜像，旧镜像不会读取它们。修改 `.env.web` 后须重新创建 Web 容器；单纯重启不会加载新的环境文件内容。

Compose 方式将同样两项写入项目 `.env`，由 `compose.yaml` 注入 Web。`WEB_IMAGE`、`WEB_PORT` 分别控制镜像和宿主机端口映射，不是 Web 进程的环境变量；当前编排映射到容器 80，因此保持 `CROWD_WEB_LISTEN=:80`。独立容器在 1Panel 直接配置镜像和端口映射。

浏览器继续使用同源 `/api/*`，由 Caddy 转发；不要把容器内部地址填到浏览器。`CROWD_API_ORIGIN` 仅用于源码开发的 Vite 代理，`VITE_API_ORIGIN` 不用于管理端运行配置。Web 不配置数据库密码、微信 AppSecret 或管理员密码。

### 等价终端操作与检查

以下命令在服务器宿主机终端执行，不是在容器终端执行。先准备上述三个环境文件并替换镜像标签；已有同名网络/容器时使用现有资源，不重复创建。

```bash
chmod 600 .env.postgres .env.api .env.web
docker network create crowd-standalone
docker run -d --name crowd-postgres --network crowd-standalone --network-alias postgres \
  --restart unless-stopped --env-file .env.postgres \
  -v crowd-standalone-postgres:/var/lib/postgresql/data postgres:17-alpine
docker exec crowd-postgres pg_isready -U crowd -d crowd
```

数据库检查成功后继续，否则先检查 `docker logs --tail 50 crowd-postgres`：

```bash
docker run -d --name crowd-api --network crowd-standalone --network-alias api \
  --restart unless-stopped --env-file .env.api \
  registry.example.com/team/crowd-api:实际版本
docker logs --tail 50 crowd-api
docker exec crowd-api curl --fail --silent --show-error http://127.0.0.1:8080/readyz
```

API 就绪成功后再启动 Web：

```bash
docker run -d --name crowd-web --network crowd-standalone --network-alias web \
  --restart unless-stopped --env-file .env.web -p 127.0.0.1:8088:80 \
  -v crowd-standalone-caddy-data:/data -v crowd-standalone-caddy-config:/config \
  registry.example.com/team/crowd-web:实际版本
curl --fail --show-error http://127.0.0.1:8088/readyz
```

如果在 1Panel 的 API 容器终端检查，只运行 `curl --fail --show-error http://127.0.0.1:8080/readyz`，无需也不能假设容器内安装了 Docker。

非空 `CROWD_DATABASE_URL` 优先于分项配置，使用分项时移除旧 URL 或设为空。保留 URL 方式时，特殊字符仍需百分号编码（如 `@` 为 `%40`）。分项主机、库名、用户名和密码均必填；端口默认 5432，SSL 模式默认 `prefer`。

## 1Panel 网站反向代理与 HTTPS

1. 域名 DNS 指向服务器，在 1Panel 创建该域名的反向代理网站，并配置有效 HTTPS 证书及 HTTP 到 HTTPS 跳转。
2. 如果入口代理运行于宿主机网络，代理目标为 `http://127.0.0.1:8088`。代理整个站点，包括 `/api/*`、`/healthz`、`/readyz` 和 SPA 页面路径；保留请求方法、Authorization、Host、Origin 与转发协议头。
3. 如果入口代理运行于独立容器，其 `127.0.0.1` 不是宿主机。让入口代理加入 Web 的 Docker 网络，改用 `http://web:80`；Compose 的 Web 服务名是 `web`，独立方案提供了相同别名。不要为此公开 PostgreSQL 或 API 端口。
4. 上传请求体上限与代理超时按应用上传需求配置，避免入口提前拒绝图片/任务证据；不要在代理缓存鉴权接口或私有附件。
5. 检查 `https://实际域名/readyz`，再通过域名首页登录管理端。小程序独立配置 HTTPS API 地址和微信合法请求/下载域名。

本手册使用 `CROWD_WEB_LISTEN=:80` 提供容器内 HTTP，公网 HTTPS 证书由 1Panel 入口管理。

## 首次安装后生成 admin 密码并初始化

如果安装时未设置初始化变量，API 可以正常就绪但不会自动创建 `admin`。无需删除数据库或重新制作镜像，按以下步骤补建：

1. 在服务器宿主机终端运行 `openssl rand -hex 24`，生成 48 字节 ASCII 随机密码，存入密码管理工具。这是后台登录密码，独立于数据库密码。也可自行设置 12–128 字节密码。
2. **Compose 方案**：在原 `.env` 中同时设置以下两项，或在 1Panel 原编排的实际环境来源中设置；不要只在临时终端 export 后以为面板会读取。

```dotenv
CROWD_BOOTSTRAP_ADMIN_USERNAME=admin
CROWD_BOOTSTRAP_ADMIN_PASSWORD=替换为刚生成并保存的密码
```

3. 在原编排中重新创建 API，使新变量进入进程。终端等价命令：

```bash
cd /opt/crowd
docker compose --env-file .env -f compose.yaml config --quiet
docker compose --env-file .env -f compose.yaml up --no-build -d --force-recreate --wait api
docker compose --env-file .env -f compose.yaml logs --tail 50 api
```

**独立容器方案**：在 1Panel 编辑 `crowd-api` 的环境变量，设置同样两项并重新创建容器，保留原镜像、数据库配置、网络及 `api` 别名。只执行 `docker restart` 或修改宿主机 `.env.api` 不会更新现有容器的环境。不要在运行中的 API 容器内再启动第二个 `crowd-api` 进程。

4. API 就绪后，打开 `https://实际域名/`，选择管理员账号密码登录，用户名 `admin`，密码为第 1 步保存的值。就绪成功只说明服务可用，需实际登录确认账号可用；启动日志不会输出密码，也不会打印 CLI 创建成功提示。
5. 登录验证成功后，从 `.env`/1Panel 环境配置中**同时移除或清空这两项**，再次重新创建 API。Compose 重复第 3 步命令，独立容器重复面板重建操作。确认旧容器已被替换，安全处理旧环境文件副本；账号与哈希保留在数据库，不因移除初始化变量而删除。不要仅移除其中一项，只有一项非空会导致启动失败。
6. 再次登录确认初始化凭据清理后账号仍可用，记录部署版本及验证结果，不在日志或工单中记录明文密码。

当前二进制**没有 `--create-admin` 命令或标准输入创建账号功能**，不要在容器终端输入密码并等待 EOF。初始化采用上述启动配置；本任务仅同步部署文档，没有新增管理 CLI。

若账号已存在，改为新的密码不会覆盖旧密码。先使用已保存的凭据；遗失时通过受控账号恢复流程处理，当前初始化机制不能当作密码重置工具，也不要删除数据卷或直接改数据库哈希。

## 持久化、更新与验收

PostgreSQL 命名卷是主要持久化数据，包含账号、业务、数字变更、图片/凭证及审计；必须备份并在隔离环境演练恢复。面板删除/重建容器时保留卷，不能用删除数据库解决初始化或密码问题。

更新版本先备份，设置新的镜像标签/digest 并拉取、重建 API/Web，等待 migration 和就绪检查成功，再检查登录、图片授权访问和业务流程。独立容器重建保留原环境、卷和网络别名。仅点击重启不会下载新镜像或重新编译源码。备份、镜像更新及迁移兼容回退要求见 [Docker 手册](docker.md)。

文档核对记录（2026-10-29）：已按 `compose.yaml`、Caddyfile、运行入口及管理员初始化实现核对配置。当前环境没有 Docker/1Panel，未实际部署、生成生产凭据、创建管理员或验证公网 HTTPS；面板操作及真实登录仍需在隔离部署环境验收。
