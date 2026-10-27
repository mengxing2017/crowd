export const ruleKinds: Record<string, string> = {
  business_chain: '业务闭环定义', operator: '平台责任主体', roles: '角色职责', completion: '任务完成口径', review: '平台审核口径', product_tasks: '产品与任务关系', attribution: '推广关系边界', digital_boundary: '数字记录边界', amount_unit: '金额单位', release_scope: '首期范围', agreements: '服务协议与隐私说明',
}
export const roleNames: Record<string, string> = {
  user: '普通用户', rule_manager: '规则管理员', identity_reviewer: '身份审核', promotion_manager: '推广管理员', auditor: '审计人员', operator: '产品运营', task_reviewer: '任务审核', settlement_reviewer: '申请审核', manual_handler: '人工经办', verifier: '复核人员', support: '客服人员',
}
export const statuses: Record<string, string> = { active: '正常', disabled: '已停用', closed: '已注销', draft: '草稿', published: '已发布', success: '成功' }
export type Section = 'welcome' | 'rules' | 'users' | 'promotions' | 'audit' | 'guide'
export const sections: { key: Section; title: string; description: string; role?: string }[] = [
  { key: 'welcome', title: '工作台', description: '从这里开始处理今天的平台业务' },
  { key: 'rules', title: '业务规则', description: '维护规则版本，保存草稿并按需发布', role: 'rule_manager' },
  { key: 'users', title: '用户与权限', description: '查询用户状态，配置岗位及数据范围', role: 'identity_reviewer' },
  { key: 'promotions', title: '推广码管理', description: '查询推广码归属，保留换发历史', role: 'promotion_manager' },
  { key: 'audit', title: '审计日志', description: '按业务对象追踪操作与处理结果', role: 'auditor' },
  { key: 'guide', title: '使用指南', description: '了解业务边界、岗位权限与操作流程' },
]
export type Run = (work: () => Promise<void>) => Promise<boolean>
export type Save = (action: string, body: Record<string, unknown>) => Promise<boolean>
