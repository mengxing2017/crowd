export type Section = 'welcome' | 'rules' | 'users' | 'promotions' | 'audit' | 'guide' | 'catalog' | 'attribution' | 'taskRules' | 'tasks'
export const sections: { key: Section; title: string; description: string; role?: string; roles?: string[] }[] = [
  { key: 'welcome', title: '工作台', description: '从这里开始处理今天的平台业务' },
  { key: 'rules', title: '业务规则', description: '维护规则版本，保存草稿并按需发布', role: 'rule_manager' },
  { key: 'users', title: '用户与权限', description: '查询用户状态，配置岗位及数据范围', role: 'identity_reviewer' },
  { key: 'promotions', title: '推广码管理', description: '查询推广码归属，保留换发历史', role: 'promotion_manager' },
  { key: 'catalog', title: '产品目录与发布', description: '管理产品草稿、独立审核、定时发布及上下架', roles: ['operator', 'product_reviewer', 'product_emergency'] },
  { key: 'attribution', title: '推广归因', description: '查看归因策略与访问统计，处理归因申诉', roles: ['rule_manager', 'attribution_reviewer'] },
  { key: 'taskRules', title: '定价与任务模板', description: '版本化佣金规则、执行步骤及参与资格', roles: ['operator', 'rule_manager', 'verifier'] },
  { key: 'tasks', title: '任务审核与质量', description: '独占审核、补交、独立复核与抽检', roles: ['task_reviewer', 'verifier'] },
  { key: 'audit', title: '审计日志', description: '按业务对象追踪操作与处理结果', role: 'auditor' },
  { key: 'guide', title: '使用指南', description: '了解业务边界、岗位权限与操作流程' },
]
