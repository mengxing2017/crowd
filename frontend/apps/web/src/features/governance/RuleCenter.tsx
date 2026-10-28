import { useState } from 'react'
import { PlusOutlined } from '@ant-design/icons'
import { Alert, Button, Card, Collapse, Drawer, Form, Input, Modal, Popconfirm, Select, Space, Table, Tag, Typography } from 'antd'
import { type Rule } from './api'
import { ruleKinds, statuses } from './model'
import { type Save } from '../platform/model'
export function RuleCenter({ rules, busy, pending, save }: { rules: Rule[]; busy: boolean; pending: boolean; save: Save }) {
  const [kind, setKind] = useState<string>()
  const [status, setStatus] = useState<string>()
  const [editing, setEditing] = useState(false)
  const [preview, setPreview] = useState<Rule>()
  const [form] = Form.useForm<{ kind: string; service?: string; privacy?: string; configuration: string }>()
  const selectedKind = Form.useWatch('kind', form) as string | undefined
  function open(rule?: Rule) {
    const nextKind = rule?.kind || 'agreements'
    form.setFieldsValue({ kind: nextKind, service: String(rule?.content.service || ''), privacy: String(rule?.content.privacy || ''), configuration: JSON.stringify(rule?.content || (nextKind === 'digital_boundary' ? { manual_only: true } : {}), null, 2) })
    setEditing(true)
  }
  return <>
    <Card><div className="filter-bar"><Space wrap><Select aria-label="筛选规则类型" placeholder="全部规则类型" allowClear options={Object.entries(ruleKinds).map(([value, label]) => ({ value, label }))} value={kind} onChange={setKind} style={{ minWidth: 200 }} /><Select aria-label="筛选规则状态" placeholder="全部状态" allowClear options={['draft', 'published', 'disabled'].map(value => ({ value, label: statuses[value] }))} value={status} onChange={setStatus} style={{ minWidth: 130 }} /></Space><Button type="primary" icon={<PlusOutlined />} disabled={busy || pending} onClick={() => open()}>新建规则草稿</Button></div>
      <Table<Rule> rowKey="id" loading={busy} scroll={{ x: 720 }} dataSource={rules.filter(rule => (!kind || rule.kind === kind) && (!status || rule.status === status)).slice().reverse()} pagination={{ pageSize: 10, showSizeChanger: true }} locale={{ emptyText: '暂无规则，请先创建草稿' }} columns={[
        { title: '规则名称', dataIndex: 'kind', render: (value: string) => ruleKinds[value] || value },
        { title: '版本', dataIndex: 'version', render: (value: number) => `V${value}` },
        { title: '状态', dataIndex: 'status', render: (value: string) => <Tag color={value === 'published' ? 'success' : value === 'draft' ? 'processing' : 'default'}>{statuses[value] || value}</Tag> },
        { title: '操作', key: 'actions', render: (_, rule) => <Space><Button type="link" onClick={() => setPreview(rule)}>查看</Button><Button type="link" disabled={busy || pending} onClick={() => open(rule)}>新建版本</Button>{rule.status !== 'disabled' && <Popconfirm title={rule.status === 'draft' ? '确认发布此规则版本？' : '确认停用此规则版本？'} description={rule.status === 'draft' ? '同类原已发布版本将被停用。' : '停用后，该版本不再用于后续业务。'} onConfirm={() => save(rule.status === 'draft' ? 'rule_publish' : 'rule_disable', { id: rule.id, version: rule.version })}><Button type="link" danger={rule.status === 'published'} disabled={busy || pending}>{rule.status === 'draft' ? '发布' : '停用'}</Button></Popconfirm>}</Space> },
      ]} /></Card>
    <Drawer title="新建规则草稿" open={editing} onClose={() => !busy && setEditing(false)} size="large" footer={<Space><Button disabled={busy} onClick={() => setEditing(false)}>取消</Button><Button type="primary" loading={busy} disabled={pending} onClick={() => form.submit()}>保存草稿</Button></Space>}>
      <Alert type="info" showIcon title="草稿不会立即生效" description="保存后请核对内容，再在规则列表单独发布。未确认的业务限制请保留为待确认项。" />
      <Form form={form} layout="vertical" disabled={busy} onFinish={async values => {
        let content: Record<string, unknown>
        if (values.kind === 'agreements') content = { service: values.service, privacy: values.privacy }
        else {
          try { const parsed: unknown = JSON.parse(values.configuration); if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error(); content = parsed as Record<string, unknown> }
          catch { form.setFields([{ name: 'configuration', errors: ['请填写有效的 JSON 对象'] }]); return }
        }
        if (await save('rule_save', { kind: values.kind, content, version: rules.filter(rule => rule.kind === values.kind).length })) setEditing(false)
      }}>
        <Form.Item name="kind" label="规则类型" rules={[{ required: true }]}><Select options={Object.entries(ruleKinds).map(([value, label]) => ({ value, label }))} /></Form.Item>
        {selectedKind === 'agreements' ? <><Form.Item name="service" label="服务协议" rules={[{ required: true, whitespace: true, message: '请填写服务协议正文' }]}><Input.TextArea rows={7} placeholder="填写经确认的服务协议正文" /></Form.Item><Form.Item name="privacy" label="隐私说明" rules={[{ required: true, whitespace: true, message: '请填写隐私说明正文' }]}><Input.TextArea rows={7} placeholder="填写资料用途、保存与访问规则" /></Form.Item></> : <><Typography.Paragraph type="secondary">结构化内容用于保留规则字段。数字边界须包含 manual_only: true；产品任务关系使用 products 数组，产品包含 id、status、tasks，任务包含 id、status、completion、evidence。</Typography.Paragraph><Form.Item name="configuration" label="规则内容（JSON 对象）" rules={[{ required: true }]}><Input.TextArea className="code-input" rows={16} /></Form.Item></>}
      </Form>
    </Drawer>
    <Modal title={preview ? `${ruleKinds[preview.kind]} · V${preview.version}` : '规则详情'} open={!!preview} onCancel={() => setPreview(undefined)} footer={<Button onClick={() => setPreview(undefined)}>关闭</Button>} width={720}>{preview && (preview.kind === 'agreements' ? <><Typography.Title level={5}>服务协议</Typography.Title><Typography.Paragraph className="preserve-lines">{String(preview.content.service || '')}</Typography.Paragraph><Typography.Title level={5}>隐私说明</Typography.Title><Typography.Paragraph className="preserve-lines">{String(preview.content.privacy || '')}</Typography.Paragraph></> : <Collapse defaultActiveKey={['content']} items={[{ key: 'content', label: '版本内容', children: <pre>{JSON.stringify(preview.content, null, 2)}</pre> }]} />)}</Modal>
  </>
}
