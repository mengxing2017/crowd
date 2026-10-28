import { useEffect, useState } from 'react'
import { Alert, Button, Card, Form, Input, InputNumber, Select, Space, Switch, Table } from 'antd'
import { withdrawals, type WithdrawalRule } from './api'
import type { User } from '../identity/api'
import type { Run, Save } from '../platform/model'
export function WithdrawalRules({ me, busy, pending, run, save }: { me: User; busy: boolean; pending: boolean; run: Run; save: Save }) {
 const [rules, setRules] = useState<WithdrawalRule[]>([])
 const [reason, setReason] = useState('')
 async function load() { const result = await withdrawals.rules(); setRules(Array.isArray(result) ? result : [result]) }
 useEffect(() => { void run(load) }, [])
 async function submit(action: string, body: Record<string, unknown>) { if (await save(action, body)) await run(load) }
 const locked = busy || pending
 return <Space orientation="vertical" className="full-width"><Alert type="info" showIcon title="规则须显式配置并独立复核后生效；不预设正式金额门槛。无额外扣减，不对接支付接口。" />
 <Table<WithdrawalRule> rowKey="id" dataSource={rules} columns={[{ title: '版本', dataIndex: 'version' }, { title: '状态', dataIndex: 'status' }, { title: '最低／最高（分）', render: (_, r) => `${r.min_cents} / ${r.max_cents}` }, { title: '收款方式', render: (_, r) => r.methods.join('、') }, { title: '登记复核', render: (_, r) => r.registration_verification ? '必须' : '不启用' }, { title: '审核', render: (_, r) => r.status === 'draft' && r.actor !== me.id && me.roles.includes('verifier') && <Space>{[true, false].map(approve => <Button key={String(approve)} disabled={locked || !reason.trim()} onClick={() => void submit('withdrawal_rule_review', { id: r.id, approve, reason })}>{approve ? '独立复核发布' : '驳回规则'}</Button>)}</Space> }]} />
 <Input placeholder="规则复核说明" value={reason} onChange={e => setReason(e.target.value)} />
 {me.roles.includes('rule_manager') && <Card title="新增规则草稿"><Form layout="vertical" onFinish={v => void submit('withdrawal_rule_propose', v)} initialValues={{ methods: ['wechat'], registration_verification: true, separate_roles: true, second_review_cents: 0, reminder_seconds: 0 }}>
 {['min_cents', 'max_cents', 'daily_count', 'daily_cents', 'lifetime_cents', 'second_review_cents', 'reminder_seconds'].map((key, index) => <Form.Item key={key} name={key} label={['最低申请（分）', '最高申请（分）', '每日次数', '每日累计申请（分）', '累计申请上限（分）', '高额独立复核起点（分，0 关闭）', '超时提醒秒数（0 关闭）'][index]} rules={[{ required: true, message: '请显式配置规则' }]}><InputNumber min={0} precision={0} /></Form.Item>)}
 <Form.Item name="methods" label="启用方式" rules={[{ required: true }]}><Select mode="multiple" options={[{ value: 'wechat', label: '微信收款二维码' }, { value: 'alipay', label: '支付宝账户（须确认资料政策）' }]} /></Form.Item>
 <Form.Item name="alipay_policy" label="支付宝资料人工检查政策（启用时必填）"><Input.TextArea /></Form.Item><Form.Item name="alipay_policy_confirmed" label="已确认支付宝资料政策" valuePropName="checked"><Switch /></Form.Item>
 <Form.Item name="registration_verification" label="付款登记须独立复核" valuePropName="checked"><Switch /></Form.Item><Form.Item name="separate_roles" label="审核、经办、结算复核岗位分离" valuePropName="checked"><Switch /></Form.Item>
 <Form.Item name="reason" label="规则依据" rules={[{ required: true }]}><Input.TextArea /></Form.Item><Alert type="info" title="日界为北京时间；申请次数及累计值包含已撤销、驳回的提交记录。发布前请核对这一口径。" /><Button htmlType="submit" disabled={locked}>保存并提交独立复核</Button>
 </Form></Card>}</Space>
}
