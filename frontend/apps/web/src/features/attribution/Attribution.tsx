import { attribution } from './api'
import { useEffect, useState } from 'react'
import { Alert, Button, Card, Form, Input, InputNumber, Modal, Select, Space, Statistic, Table, Tag } from 'antd'
import { type AttributionAdmin, type AttributionDispute } from './api'
import { type User } from '../identity/api'
import type { Run, Save } from '../platform/model'
export function Attribution({ me, busy, pending, run, save }: { me: User; busy: boolean; pending: boolean; run: Run; save: Save }) {
  const [data, setData] = useState<AttributionAdmin>()
  const [dispute, setDispute] = useState<AttributionDispute>()
  const [form] = Form.useForm()
  const locked = busy || pending
  const reviewer = me.roles.includes('attribution_reviewer')
  async function load() { setData(await (reviewer ? attribution.admin : attribution.policies)(me.scope.includes('*') ? '*' : me.scope[0])) }
  useEffect(() => { void run(load) }, [])
  async function submit(action: string, body: Record<string, unknown>) { if (await save(action, body)) { await run(load); return true } return false }
  return <Space orientation="vertical" size={20} className="full-width"><Alert type="info" showIcon title="扫码、领取与完成是不同事件" description="触点仅记录访问；领取锁定归因。申诉裁定保留原触点，不直接转移任何人的佣金数字。完成事件接口需可信来源适配器接入，当前不开放外部回传。" />
    {reviewer && <Card title="访问与策略" extra={<Button disabled={busy} onClick={() => void run(load)}>刷新归因数据</Button>}><Statistic title="已记录访问次数" value={data?.touch_count || 0} /><Table rowKey="id" dataSource={data?.policies || []} columns={[{ title: '版本', dataIndex: 'version' }, { title: '窗口（秒）', dataIndex: 'window_seconds' }, { title: '策略', render: (_, p) => p.strategy === 'first_touch' ? '首触' : '末触' }]} /></Card>}
    {me.roles.includes('rule_manager') && <Card title="发布归因策略"><Alert title="窗口从访问触点服务端时间起算，包含结束边界；策略只影响后续领取，已领取快照不变。请明确填写业务确认参数。" type="warning" /><Form layout="vertical" disabled={locked || !data} onFinish={values => submit('policy_publish', { ...values, version: data?.policies.length || 0 })}><Form.Item name="window_seconds" label="归因有效窗口（秒）" rules={[{ required: true }]}><InputNumber min={1} precision={0} max={Number.MAX_SAFE_INTEGER} /></Form.Item><Form.Item name="strategy" label="多触点策略" rules={[{ required: true }]}><Select options={[{ value: 'first_touch', label: '首触' }, { value: 'last_touch', label: '末触' }]} /></Form.Item><Button htmlType="submit" type="primary">发布新策略版本</Button></Form></Card>}
    {reviewer && <Card title="归因争议"><Table rowKey="id" dataSource={data?.disputes || []} columns={[{ title: '任务实例', dataIndex: 'claim_id' }, { title: '理由', dataIndex: 'reason' }, { title: '证据', dataIndex: 'evidence' }, { title: '状态', render: (_, d) => <Tag>{d.status === 'pending' ? '待裁定' : d.status === 'uphold' ? '维持归属' : '待关联调整核查'}</Tag> }, { title: '操作', render: (_, d) => d.status === 'pending' && d.owner !== me.id && <Button disabled={locked} onClick={() => { setDispute(d); form.resetFields() }}>裁定</Button> }]} /></Card>}
    <Modal title="归因争议裁定" open={!!dispute} onCancel={() => setDispute(undefined)} footer={null}><Form form={form} layout="vertical" disabled={locked} onFinish={async values => { if (dispute && await submit('dispute_resolve', { ...values, id: dispute.id, version: dispute.version })) setDispute(undefined) }}><Form.Item name="decision" label="裁定结果" rules={[{ required: true }]}><Select options={[{ value: 'uphold', label: '维持原归属' }, { value: 'needs_adjustment', label: '转关联数字调整核查（不直接改数字）' }]} /></Form.Item><Form.Item name="reason" label="证据核查与调整依据" rules={[{ required: true }]}><Input.TextArea /></Form.Item><Button htmlType="submit" type="primary">保存裁定记录</Button></Form></Modal>
  </Space>
}
