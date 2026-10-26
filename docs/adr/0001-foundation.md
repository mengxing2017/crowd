# 0001 工程骨架

工程采用两部署应用结构：Rust API 与 React Web；PostgreSQL 为独立基础服务。uni-app 小程序为独立编译入口，api-contract 为不部署的平台中立协议包。

只保留 system 技术模块和工程元数据迁移，不预设业务模型、身份系统或第三方集成。请求 transport 由各端注入；今后通过 feature/api 接入统一契约。数据库就绪探针通过南向端口实现，领域和用例不依赖 HTTP 框架。
