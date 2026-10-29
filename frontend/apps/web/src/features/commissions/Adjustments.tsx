import { useState } from 'react'
import { Alert, Button, Card, Input, Space, Table } from 'antd'
import { commissions } from './api'
import { OperationForm } from '../../shared/OperationForm'
import type { Run, Save } from '../platform/model'
type Row = Record<string, unknown>
export function Adjustments({ locked, run, save }: { locked: boolean; run: Run; save: Save }) {
 const [owner, setOwner] = useState(''); const [rows, setRows] = useState<Row[]>([]); const [selected, setSelected] = useState<Row[]>([]); const [batch, setBatch] = useState<Row>()
 const [reason, setReason] = useState(''); const [locating, setLocating] = useState<Row>()
 async function load() { setRows(await commissions.adjustments(owner)) }
 async function submit(a: string, body: Row) { if (await save(a, body)) await run(load) }
 return <Space orientation="vertical" className="full-width"><Alert type="warning" title="任务撤销与数字调整保留原任务、依据和流水。已占用或已登记结算部分先核查，不自动追回，也不抵扣未来佣金。" /><Space><Input placeholder="用户编号" value={owner} onChange={e => setOwner(e.target.value)} /><Button disabled={locked || !owner} onClick={() => void run(load)}>查询该用户调整单</Button></Space>
 <Card title="登记失效、退款说明或恢复依据"><OperationForm locked={locked} label="提交独立复核" fields={[{ name: 'task_id', label: '原任务实例编号' }, { name: 'basis_key', label: '唯一依据事件号' }, { name: 'source', label: '业务来源' }, { name: 'evidence', label: '可信业务依据说明', kind: 'text' }, { name: 'amount_cents', label: '调减或恢复数字（正整数分，待审取消填0）', kind: 'integer' }, { name: 'invalid_base_cents', label: '部分失效基数（可选，按原可信基数与舍入规则计算）', kind: 'integer', optional: true }, { name: 'restores', label: '原调减单编号（仅恢复时填写）', optional: true }, { name: 'ticket_id', label: '关联申诉工单（可选）', optional: true }, { name: 'reason', label: '用户可见原因', kind: 'text' }]} submit={v => submit('adjustment_propose', { ...v, owner })} /></Card>
 <Input placeholder="独立复核说明" value={reason} onChange={e => setReason(e.target.value)} />
 <Table<Row> rowKey={r => String(r.id)} dataSource={rows} rowSelection={{ onChange: (_, r) => setSelected(r), getCheckboxProps: r => ({ disabled: !['pending', 'awaiting_resolution'].includes(String(r.status)) }) }} columns={[{ title: '调整单', dataIndex: 'id' }, { title: '原任务', dataIndex: 'task_id' }, { title: '方向', render: (_, r) => r.direction === 'restore' ? '恢复' : '调减／待审取消' }, { title: '整数分', dataIndex: 'amount_cents' }, { title: '依据', dataIndex: 'reason' }, { title: '状态', dataIndex: 'status' }, { title: '复核', render: (_, r) => r.status === 'pending_location' ? <Button onClick={() => setLocating(r)}>补充原任务定位</Button> : r.kind === 'batch' ? <Button onClick={() => setBatch(r)}>查看批次预览</Button> : ['pending', 'awaiting_resolution'].includes(String(r.status)) && <Space>{[true,false].map(approve => <Button disabled={locked || !reason} key={String(approve)} onClick={() => void submit('adjustment_review', { id: r.id, version: r.version, approve, reason })}>{approve ? '复核生效或进入核查' : '驳回'}</Button>)}</Space> }]} />
 {locating && <Card title="补充可信任务定位"><OperationForm locked={locked} label="定位后进入独立复核" fields={[{ name: 'task_id', label: '可信原任务实例编号' }, { name: 'reason', label: '定位依据' }]} submit={v => submit('adjustment_locate', { ...v, id: locating.id, version: locating.version })} /></Card>}
 <Button disabled={locked || !selected.length || !reason} onClick={() => void run(async () => { if (await save('adjustment_batch_preview', { items: selected.map(r => ({ id: r.id, version: r.version })), reason })) { const all = await commissions.adjustments(owner); setBatch(all.filter(r => r.kind === 'batch').at(-1)); setRows(all) } })}>预览所选批次影响</Button>
 {batch && <Card title="批量复核预览"><p>影响人数：{String(batch.affected_users)}；计划调整整数分：{String(batch.amount_cents)}；在途或已登记申请：{(batch.applications as Row[]).map(a => `${a.id}（${a.status}）`).join('、') || '无'}</p><Button disabled={locked || !reason} onClick={() => void submit('adjustment_batch_review', { id: batch.id, version: batch.version, confirmed: true, reason })}>由独立复核人确认原批次</Button></Card>}
 </Space>
}
