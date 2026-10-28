import { promotion } from './api'
import { useState } from 'react'
import { Alert, Button, Card, Form, Input, Modal, Space, Table, Tag, Typography } from 'antd'
import { SearchOutlined } from '@ant-design/icons'
import { type Promotion } from './api'
import type { Run, Save } from '../platform/model'
export function Promotions({ busy, pending, run, save }: { busy: boolean; pending: boolean; run: Run; save: Save }) {
  const [rows, setRows] = useState<Promotion[] | null>(null)
  const [owner, setOwner] = useState('')
  const [selected, setSelected] = useState<Promotion>()
  const [form] = Form.useForm()
  async function load(target: string) { setRows(null); setOwner(target); setRows(await promotion.managedCodes(target)) }
  return <Space orientation="vertical" size={24} className="full-width"><Card title="推广码归属查询"><Form layout="inline" onFinish={values => { void run(() => load(String(values.target).trim())) }}><Form.Item name="target" label="所属用户" rules={[{ required: true, whitespace: true, message: '请输入用户编号' }]}><Input placeholder="输入完整用户编号" style={{ width: 320, maxWidth: '100%' }} /></Form.Item><Form.Item><Button type="primary" htmlType="submit" icon={<SearchOutlined />} loading={busy}>查询推广码</Button></Form.Item></Form></Card>
    <Alert showIcon type="info" title="个人推广码与收款二维码分别管理" description="推广码保持唯一归属，撤销换发保留历史。扫码只记录入口，不代表任务完成或可申请佣金。" />
    <Card title="推广码列表"><Table<Promotion> rowKey="id" loading={busy} dataSource={rows || []} scroll={{ x: 760 }} locale={{ emptyText: rows === null ? '请先查询所属用户' : '该用户暂无推广码' }} columns={[
      { title: '推广码', dataIndex: 'id', render: (id: string) => <Typography.Text copyable>{id}</Typography.Text> },
      { title: '类型', key: 'type', render: (_, row) => row.code.task ? '任务定向码' : row.code.product ? '产品定向码' : '个人主码' },
      { title: '产品 / 任务', key: 'target', render: (_, row) => [row.code.product, row.code.task].filter(Boolean).join(' / ') || '不限' },
      { title: '版本', key: 'version', render: (_, row) => `V${row.code.version}` },
      { title: '状态', key: 'active', render: (_, row) => <Tag color={row.code.active ? 'success' : 'default'}>{row.code.active ? '有效' : '已撤销'}</Tag> },
      { title: '操作', key: 'action', render: (_, row) => <Button type="link" disabled={!row.code.active || busy || pending} onClick={() => { form.resetFields(); setSelected(row) }}>撤销并换发</Button> },
    ]} /></Card>
    <Modal title="撤销并换发推广码" open={!!selected} onCancel={() => !busy && setSelected(undefined)} confirmLoading={busy} okText="确认换发" okButtonProps={{ disabled: pending }} onOk={() => form.submit()}><Alert type="warning" showIcon title="原推广码将失效" description="原码历史归属继续保留，换发不会覆盖历史注册来源。" /><Form form={form} layout="vertical" disabled={busy} onFinish={values => { if (!selected) return; void (async () => { if (await save('code_rotate', { ...values, id: selected.id, version: selected.code.version })) { setSelected(undefined); await run(() => load(owner)) } })() }}><Form.Item label="原推广码"><Typography.Text copyable>{selected?.id}</Typography.Text></Form.Item><Form.Item name="reason" label="换发原因" rules={[{ required: true, whitespace: true, message: '请填写换发原因' }]}><Input.TextArea rows={3} /></Form.Item><Form.Item name="approval" label="审批依据" rules={[{ required: true, whitespace: true, message: '请填写审批依据或关联审批编号' }]}><Input placeholder="填写已确认的审批依据" /></Form.Item></Form></Modal>
  </Space>
}
