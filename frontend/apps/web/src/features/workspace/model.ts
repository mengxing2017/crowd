export type Section = 'security' | 'reporting' | 'privacy' | 'campaigns' | 'adjustments' | 'support' | 'risk' | 'notifications' | 'commissions' | 'withdrawalRules' | 'settlements' | 'welcome' | 'rules' | 'users' | 'promotions' | 'audit' | 'guide' | 'catalog' | 'attribution' | 'taskRules' | 'tasks'
export const sections: { key: Section; title: string; description: string; role?: string; roles?: string[] }[] = [
 { key: 'security', title: '后台安全与临时授权', description: '再次验证、独立授权审批与任务来源启停', role: 'identity_reviewer' },
 { key: 'reporting', title: '统计与记录核对', description: '固定水位、安全导出与差异处置', roles: ['auditor', 'operator'] },
 { key: 'privacy', title: '隐私与数字治理', description: '显式保留政策、注销审批与修复预检', roles: ['privacy_manager', 'rule_manager', 'settlement_reviewer', 'verifier'] },
  { key: 'campaigns', title: '活动计划与统计', description: '可选计划额度、业务统计与存量责任', roles: ['operator', 'auditor'] },
  { key: 'adjustments', title: '任务撤销与数字调整', description: '依据登记、独立复核与在途争议', roles: ['settlement_reviewer', 'verifier', 'auditor'] },
  { key: 'support', title: '客服工单与申诉', description: '分类咨询、转派升级与独立复核', roles: ['support', 'rule_manager'] },
  { key: 'risk', title: '风险核查', description: '版本化规则、人工处置与误判解除', role: 'risk_manager' },
  { key: 'notifications', title: '消息与岗位待办', description: '站内告知、受控目标与模板管理' },
  { key: 'welcome', title: '工作台', description: '从这里开始处理今天的平台业务' },
  { key: 'rules', title: '业务规则', description: '维护规则版本，保存草稿并按需发布', role: 'rule_manager' },
  { key: 'users', title: '用户与权限', description: '查询用户状态，配置岗位及数据范围', role: 'identity_reviewer' },
  { key: 'promotions', title: '推广码管理', description: '查询推广码归属，保留换发历史', role: 'promotion_manager' },
  { key: 'catalog', title: '产品目录与发布', description: '管理产品草稿、独立审核、定时发布及上下架', roles: ['operator', 'product_reviewer', 'product_emergency'] },
  { key: 'attribution', title: '推广归因', description: '查看归因策略与访问统计，处理归因申诉', roles: ['rule_manager', 'attribution_reviewer'] },
  { key: 'taskRules', title: '定价与任务模板', description: '版本化佣金规则、执行步骤及参与资格', roles: ['operator', 'rule_manager', 'verifier'] },
  { key: 'tasks', title: '任务审核与质量', description: '独占审核、补交、独立复核与抽检', roles: ['task_reviewer', 'verifier'] },
  { key: 'commissions', title: '佣金数字与流水', description: '核对数字来源、调整复核及争议暂缓', roles: ['settlement_reviewer', 'verifier', 'auditor'] },
  { key: 'withdrawalRules', title: '提现申请规则', description: '显式配置门槛及独立复核政策', roles: ['rule_manager', 'verifier'] },
  { key: 'settlements', title: '提现审核与人工结算', description: '独占经办、线下付款登记及核实复核', roles: ['settlement_reviewer', 'manual_handler', 'verifier', 'auditor'] },
  { key: 'audit', title: '审计日志', description: '按业务对象追踪操作与处理结果', role: 'auditor' },
  { key: 'guide', title: '使用指南', description: '了解业务边界、岗位权限与操作流程' },
]

export const sectionGroups: { key: string; title: string; icon: Section; sections: Section[] }[] = [
  { key: 'overview', title: '工作台与消息', icon: 'welcome', sections: ['welcome', 'notifications', 'guide'] },
  { key: 'promotion', title: '产品与推广', icon: 'catalog', sections: ['catalog', 'campaigns', 'promotions', 'attribution'] },
  { key: 'execution', title: '任务管理', icon: 'tasks', sections: ['taskRules', 'tasks'] },
  { key: 'settlement', title: '佣金与人工结算', icon: 'commissions', sections: ['commissions', 'withdrawalRules', 'settlements', 'adjustments'] },
  { key: 'service', title: '客服与风险', icon: 'support', sections: ['support', 'risk'] },
  { key: 'administration', title: '系统与治理', icon: 'security', sections: ['users', 'security', 'rules', 'privacy', 'reporting', 'audit'] },
]
