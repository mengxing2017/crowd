import { Alert, Card, Collapse, Space, Steps, Table, Typography } from 'antd'
import { roleNames } from '../../identity/model'
const responsibilities: Record<string, string> = { operator: '维护产品和活动、查看授权统计，数字额度不代表资金', task_reviewer: '独占领取任务审核、按原提交版本核查证据', settlement_reviewer: '核查数字来源及收款快照，通过后仅进入待人工处理', manual_handler: '独占经办、交接和待核实；线下手动付款后登记原结果', verifier: '独立复核规则、数字修正及人工登记，不复核本人经办记录', support: '受理工单及申诉，不直接修改佣金数字', privacy_manager: '按已发布保留政策处理副本与注销，保护未结数字及证据', rule_manager: '维护业务规则、保存草稿、发布与停用版本', identity_reviewer: '查询用户状态、配置岗位与数据范围、设置业务限制', promotion_manager: '按所属用户查询推广码，经审批后撤销并换发', auditor: '按对象查询操作、请求编号、规则版本与结果' }
export function Guide() {
  return <Space orientation="vertical" size={24} className="full-width"><Card title="欢迎使用 Crowd 运营管理平台"><Typography.Paragraph>这里集中管理业务规则、用户岗位与推广记录。左侧导航根据当前岗位展示入口；每次查询和提交仍由服务端检查身份、对象范围与状态。</Typography.Paragraph><Alert type="info" showIcon title="按岗位办理任务、数字审核与人工结算" description="身份与资料、推广归因、产品发布、独立定价与模板审批、任务执行、私有证据提交、审核及质量复核、内部记录核对、工单和隐私治理。审核确认衔接佣金数字流水及可申请汇总；提现规则须独立批准，申请审核、独占人工经办、付款登记及复核分别处理。" /></Card>
    <Card title="首次使用"><Steps orientation="vertical" items={[{ title: '登录并核对岗位', content: '使用管理员账号密码登录，或兑换小程序生成的 60 秒一次性票据。刷新页面会恢复当前标签页会话并重新校验权限；会话过期后需重新登录。' }, { title: '发布协议与业务规则', content: '在业务规则创建草稿，填写服务协议与隐私说明，核对后单独发布。新版本发布会停用同类旧版本。' }, { title: '按对象办理业务', content: '在用户或推广码页面填写完整用户编号查询。变更前核对对象、版本与操作原因。' }, { title: '核查结果与记录', content: '提交结果不明时，先使用页面提示查询原结果；重试沿用原请求内容和请求键。通过审计日志核对操作。' }]} /></Card>
    <Card title="岗位与权限"><Table pagination={false} rowKey="role" dataSource={Object.entries(responsibilities).map(([role, description]) => ({ role, description }))} columns={[{ title: '岗位', dataIndex: 'role', render: (role: string) => roleNames[role] }, { title: '当前可用操作', dataIndex: 'description' }]} /></Card>
    <Card title="常见问题"><Collapse items={[
      { key: 'money', label: '页面中的佣金数字是否代表账户余额？', children: '不是。佣金为业务标记数字，不是存款、真实钱包余额或托管资金。实际付款由人员在系统外手动完成。' },
      { key: 'receipt', label: '为什么后台不能直接查看完整收款资料？', children: '微信仅上传本人收款二维码，支付宝仅填写收款账户。完整资料查看须有申请范围和用途授权，在申请详情填写查看用途后，按本单岗位与范围授权查看；普通资料列表不返回完整账户或图片。' },
      { key: 'promotion', label: '换发推广码会修改历史归属吗？', children: '不会。旧码撤销后保留历史归属和注册来源，新码保持原所属用户。扫码本身不能产生可申请佣金。' },
      { key: 'permission', label: '看不到菜单或操作被拒绝怎么办？', children: '联系权限管理员核对岗位及数据范围。页面隐藏不是最终授权，服务端仍会拒绝越权访问；不能修改自己的权限。' },
      { key: 'timeout', label: '提交超时后能重新提交吗？', children: '先查询原结果。如果需要重试，使用页面上的原请求重试，不重复发起新的业务操作。退出账号会清理当前账号的待确认引用。' },
    ]} /></Card>
  </Space>
}
