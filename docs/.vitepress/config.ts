import { defineConfig } from 'vitepress'

export default defineConfig({
  markdown: { config(md) {
    const render = md.renderer.rules.link_open || ((tokens, idx, options, _env, self) => self.renderToken(tokens, idx, options));
    md.renderer.rules.link_open = (tokens, idx, options, env, self) => {
      const href = tokens[idx].attrGet('href');
      if (href?.startsWith('../docs/')) tokens[idx].attrSet('href', href.replace('../docs/', '../'));
      return render(tokens, idx, options, env, self);
    };
  } },
  "lang": "zh-CN",
  "title": "Crowd",
  "description": "产品推广、任务执行、佣金数字与人工结算记录平台文档",
  "cleanUrls": true,
  "themeConfig": {
    "nav": [
      {
        "text": "指南",
        "link": "/guide/"
      },
      {
        "text": "需求",
        "link": "/requirements/"
      },
      {
        "text": "架构",
        "link": "/adr/"
      },
      {
        "text": "运行",
        "link": "/runbooks/"
      }
    ],
    "sidebar": [
      {
        "text": "开始阅读",
        "items": [
          {
            "text": "文档首页",
            "link": "/"
          },
          {
            "text": "项目与开发指南",
            "link": "/guide/"
          },
          {
            "text": "文档站使用与维护",
            "link": "/guide/documentation"
          },
          {
            "text": "API 契约",
            "link": "/reference/api-contract"
          }
        ]
      },
      {
        "text": "需求追踪",
        "collapsed": false,
        "items": [
          {
            "text": "需求追踪索引",
            "link": "/requirements/"
          },
          {
            "text": "四份方案需求追踪",
            "link": "/requirements/m01-m04"
          },
          {
            "text": "M05–M07 需求与实现追踪",
            "link": "/requirements/m05-m07"
          },
          {
            "text": "M08–M12 实现与验收追踪",
            "link": "/requirements/m08-m12"
          },
          {
            "text": "M13–M16 实现与验证追踪",
            "link": "/requirements/m13-m16"
          },
          {"text": "M17–M19 收款资料与人工登记追踪", "link": "/requirements/m17-m19"},
          {"text": "M20–M24 数字调整与用户服务", "link": "/requirements/m20-m24"}
        ]
      },
      {
        "text": "架构决策",
        "collapsed": true,
        "items": [
          {
            "text": "架构决策索引",
            "link": "/adr/"
          },
          {
            "text": "0001 工程骨架",
            "link": "/adr/0001-foundation"
          },
          {
            "text": "M01–M04 实现决策",
            "link": "/adr/0002-identity-profile-promotion"
          },
          {
            "text": "ADR 0003：产品发布快照、访问归因与独立领取",
            "link": "/adr/0003-catalog-attribution-lifecycle"
          },
          {
            "text": "ADR 0004：任务快照、私有证据与审核数字确认",
            "link": "/adr/0004-task-pricing-evidence-review"
          },
          {
            "text": "ADR 0005：契约包按业务领域组织",
            "link": "/adr/0005-contract-domain-boundaries"
          },
          {
            "text": "ADR 0006：Rust 模块入口只声明与导出",
            "link": "/adr/0006-rust-module-entrypoints"
          },
          {
            "text": "ADR 0007：HTTP remote 按实体和用例拆分",
            "link": "/adr/0007-http-remote-responsibilities"
          },
          {
            "text": "ADR 0008：local 应用服务按用例拆分",
            "link": "/adr/0008-local-application-use-cases"
          },
          {
            "text": "ADR 0009：拆除 community 的混合业务归属",
            "link": "/adr/0009-community-domain-ownership"
          },
          {
            "text": "ADR 0010：契约与两端前端同步领域归属",
            "link": "/adr/0010-contract-frontend-domain-ownership"
          },
          {
            "text": "ADR 0011：佣金数字、提现申请与人工结算",
            "link": "/adr/0011-commission-withdrawal-manual-settlement"
          },
          {"text": "ADR 0012：收款资料确认与人工异常处理", "link": "/adr/0012-receipt-management-manual-exceptions"},
          {"text": "ADR 0015：运营领域与用户告知事务", "link": "/adr/0015-operations-domain-workflows"},
          {"text": "ADR 0013：领域依赖隔离", "link": "/adr/0013-domain-north-south-ports"},
          {"text": "ADR 0014：领域入口与实际事件处理", "link": "/adr/0014-domain-public-entrypoints"}
        ]
      },
      {
        "text": "运行手册",
        "collapsed": false,
        "items": [
          {
            "text": "运行手册索引",
            "link": "/runbooks/"
          },
          {"text": "Docker 镜像构建、发布与部署", "link": "/runbooks/docker"},
          {"text": "1Panel 部署与首次管理员初始化", "link": "/runbooks/1panel"},
          {
            "text": "嵌入式 PostgreSQL 开发与验证",
            "link": "/runbooks/embedded-postgres"
          },
          {
            "text": "骨架验证",
            "link": "/runbooks/foundation-validation"
          },
          {
            "text": "M01–M04 运行与验证",
            "link": "/runbooks/m01-m04"
          },
          {
            "text": "M05–M07 开发与验证",
            "link": "/runbooks/m05-m07"
          },
          {
            "text": "M08–M12 运行与验证",
            "link": "/runbooks/m08-m12"
          },
          {
            "text": "M13–M16 运行与验证记录",
            "link": "/runbooks/m13-m16"
          },
          {"text": "M17–M19 收款资料与人工异常运行手册", "link": "/runbooks/m17-m19"},
          {"text": "M20–M24 数字调整与客服风险操作", "link": "/runbooks/m20-m24"},
          {
            "text": "Web 管理端重构与验证",
            "link": "/runbooks/web-console"
          }
        ]
      }
    ],
    "search": {
      "provider": "local"
    },
    "outline": {
      "label": "本页目录",
      "level": [
        2,
        3
      ]
    },
    "docFooter": {
      "prev": "上一篇",
      "next": "下一篇"
    },
    "lastUpdated": {
      "text": "最后更新"
    },
    "footer": {
      "message": "MIT 许可证 · 实现和自动化验证不等于业务验收。佣金为业务数字，付款由人员在线下手动完成。"
    }
  }
})
