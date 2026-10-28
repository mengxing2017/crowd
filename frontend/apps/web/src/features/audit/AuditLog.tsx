import { audit, type AuditRecord } from './api'
import { useState } from 'react'
import { Button, Card, Drawer, Form, Input, Table, Tag, Typography } from 'antd'
import { SearchOutlined } from '@ant-design/icons'
import { statuses } from './model'
import { type Run } from '../platform/model'
export function AuditLog({ busy, run }: { busy: boolean; run: Run }) {
  const [rows, setRows] = useState<AuditRecord[] | null>(null)
  const [selected, setSelected] = useState<AuditRecord>()
  return <><Card title="查询审计记录"><Form layout="inline" onFinish={values => { setRows(null); void run(async () => { setRows(await audit.records(String(values.target).trim())) }) }}><Form.Item name="target" label="业务对象" rules={[{ required: true, whitespace: true, message: '请输入业务对象编号' }]}><Input placeholder="对象编号，或授权范围内的 *" style={{ width: 320, maxWidth: '100%' }} /></Form.Item><Form.Item><Button type="primary" htmlType="submit" icon={<SearchOutlined />} loading={busy}>查询日志</Button></Form.Item></Form><Typography.Paragraph type="secondary" className="section-note">按时间倒序展示最近 100 条授权记录。查询不会扩大当前账号的数据范围。</Typography.Paragraph></Card><Card title="操作记录" className="stack-card"><Table<AuditRecord> rowKey={(_, index) => String(index)} loading={busy} dataSource={rows || []} scroll={{ x: 960 }} locale={{ emptyText: rows === null ? '请先选择业务对象并查询' : '暂无符合条件的审计记录' }} columns={[
    { title: '操作时间', dataIndex: 'time', width: 200, render: (time: string) => new Date(time).toLocaleString('zh-CN') },
    { title: '操作者', dataIndex: 'actor', ellipsis: true }, { title: '动作', dataIndex: 'action' }, { title: '业务对象', dataIndex: 'object', ellipsis: true },
    { title: '结果', dataIndex: 'result', render: (result: string) => <Tag color={result === 'success' ? 'success' : 'error'}>{statuses[result] || result}</Tag> },
    { title: '操作', key: 'detail', render: (_, row) => <Button type="link" onClick={() => setSelected(row)}>查看详情</Button> },
  ]} /></Card><Drawer title="审计记录详情" open={!!selected} onClose={() => setSelected(undefined)} size="large"><Typography.Paragraph type="secondary">原始记录包含请求编号、规则版本与结果，保留供核查。</Typography.Paragraph><pre>{JSON.stringify(selected, null, 2)}</pre></Drawer></>
}
