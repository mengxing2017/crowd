---
layout: home
hero:
  name: Crowd
  text: 项目文档中心
  tagline: 产品推广 · 任务执行 · 佣金数字 · 人工结算记录
  actions:
    - theme: brand
      text: 开发与使用
      link: /guide/
    - theme: alt
      text: 运行手册
      link: /runbooks/
features:
  - title: 需求可追踪
    details: M01–M24 分阶段追踪需求、实现、依赖与验证证据。
    link: /requirements/
  - title: 领域有边界
    details: 后端、API 契约与前端按领域组织，记录兼容协议与事务决策。
    link: /adr/
  - title: 操作有依据
    details: 嵌入式 PostgreSQL、管理工作台、岗位操作与隔离验证入口。
    link: /runbooks/
---

## 阅读路线

| 目标 | 入口 |
| --- | --- |
| 首次运行项目、了解范围 | [项目与开发指南](./guide/index.md) |
| 启动、构建及维护文档 | [文档站使用与维护](./guide/documentation.md) |
| 查看需求与实现对应关系 | [需求追踪](./requirements/index.md) |
| 理解模块拆分与事务边界 | [架构决策](./adr/index.md) |
| 配置提现规则、执行人工登记 | [M13–M16 运行手册](./runbooks/m13-m16.md) |
| 接入客户端 | [API 契约](./reference/api-contract.md) |

## 当前状态与边界

当前代码包含 M01–M16 实现，具体能力、依赖和未验证项以各阶段追踪表及运行记录为准。历史文档保留原实施时点，不代表当前所有功能均已业务验收。

佣金是平台记录的业务数字，不是存款或托管资金。实际付款由人员在线下手动完成；系统不调用微信／支付宝支付、转账或查单接口。审核通过仅进入待人工处理，结果不明保留申请占用。
