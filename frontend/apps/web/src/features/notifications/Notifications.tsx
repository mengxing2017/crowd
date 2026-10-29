import { useState } from 'react'
import { Alert, Button, Card, Space, Table } from 'antd'
import { notifications, type NotificationsRecord } from './api'
import { OperationForm } from '../../shared/OperationForm'
import type { Run, Save } from '../platform/model'
export function Notifications({ locked, run, save }: { locked: boolean; run: Run; save: Save }) {
 const [rows, setRows] = useState<NotificationsRecord[]>([]); const [todos, setTodos] = useState<NotificationsRecord[]>([])
 async function load() { setRows(await notifications.messages()); setTodos(await notifications.notificationTodos()) }
 async function submit(a: string, v: Record<string, unknown>) { if (await save(a, v)) await run(load) }
 return <Space orientation="vertical" className="full-width"><Alert type="info" title="站内通知关联真实单据，跳转后以当前业务状态为准。外部订阅渠道未配置，不宣称微信消息发送成功；拒绝订阅不影响查询。" /><Button disabled={locked} onClick={() => void run(load)}>刷新本人消息与岗位待办</Button>
 <Table<NotificationsRecord> rowKey="id" dataSource={todos} columns={[{ title: '待办业务', dataIndex: 'id' }, { title: '类型', dataIndex: 'kind' }, { title: '当前状态', dataIndex: 'status' }]} />
 <Table<NotificationsRecord> rowKey="id" dataSource={rows} columns={[{ title: '业务编号', dataIndex: 'business_id' }, { title: '通知状态', dataIndex: 'status' }, { title: '当前状态', dataIndex: 'current_status' }, { title: '说明', dataIndex: 'notice' }, { title: '操作', render: (_, r) => <Space><Button onClick={() => { window.location.hash = `#/${r.target === 'withdrawals' ? 'settlements' : r.target}` }}>查看当前业务</Button>{!r.read_at && <Button disabled={locked} onClick={() => void submit('notification_read', { id: r.id, version: r.version })}>标记已读</Button>}</Space> }]} />
 <Card title="运营模板预览与版本管理"><OperationForm locked={locked} label="发布预览后的模板" fields={[{ name: 'id', label: '模板编号' }, { name: 'version', label: '当前版本（新建为0）', kind: 'integer' }, { name: 'title', label: '标题' }, { name: 'body', label: '模板正文（必须保留 {business_id}、{status}、{notice}）', kind: 'text' }, { name: 'variables', label: '变量清单', kind: 'tags' }, { name: 'target', label: '受控目标页面', kind: 'choice', options: ['tasks','commissions','withdrawals','support','risk'].map(value => ({ value, label: value })) }, { name: 'reminder_seconds', label: '临期提醒窗口（秒，可选）', kind: 'integer', optional: true }, { name: 'max_attempts', label: '外部通知最大尝试次数（启用渠道时填写）', kind: 'integer', optional: true }, { name: 'retry_seconds', label: '通知重试间隔（秒）', kind: 'integer', optional: true }, { name: 'preview_confirmed', label: '已核对变量、必要说明及目标页面', kind: 'boolean' }, { name: 'reason', label: '发布依据' }]} submit={v => submit('notification_template', v)} /></Card>
 </Space>
}
