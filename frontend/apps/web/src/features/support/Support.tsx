import { useState } from 'react'
import { Alert, Button, Card, Input, Select, Space, Table, Upload } from 'antd'
import { support } from './api'
import { OperationForm } from '../../shared/OperationForm'
import { uploadSupportAttachment, viewPrivateImage } from '../files/api'
import type { Run, Save } from '../platform/model'
type Row = Record<string, unknown>
export function Support({ locked, run, save }: { locked: boolean; run: Run; save: Save }) {
 const [rows, setRows] = useState<Row[]>([]); const [selected, select] = useState<Row>(); const [reply, setReply] = useState(''); const [internal, setInternal] = useState(false); const [attachments, setAttachments] = useState<string[]>([]); const [image, setImage] = useState(''); const [statistics, setStatistics] = useState<Row>()
 async function load() { const r = await support.tickets(); setRows(r); if (selected) select(r.find(t => t.id === selected.id)) }
 async function submit(action: string, body: Row) { if (await save(action, body)) { setReply(''); setAttachments([]); await run(load) } }
 function closeImage() { if (image) URL.revokeObjectURL(image); setImage('') }
 return <Space orientation="vertical" className="full-width"><Alert type="info" title="客服工单与申诉：关联本人单据、转派升级、独立复核。内部备注不会发送给用户；结案不直接修改佣金或承诺线下到账。" /><Space><Button disabled={locked} onClick={() => void run(load)}>刷新授权范围内工单</Button><Button disabled={locked} onClick={() => void run(async () => setStatistics(await support.supportStatistics()))}>服务质量统计</Button></Space>
 {statistics && <Card title="可追溯服务统计"><p>工单总数 {String(statistics.total)}；结案 {String(statistics.closed)}；重复咨询集合 {String(statistics.repeated_complaint_groups)}。</p><p>统计集合：{(statistics.ticket_ids as string[]).join('、') || '暂无'}</p><p>响应时长（秒）：{(statistics.response_seconds as number[]).join('、') || '暂无已回复工单'}；解决时长（秒）：{(statistics.resolution_seconds as number[]).join('、') || '暂无结案'}</p></Card>}
 <Table<Row> rowKey={r => String(r.id)} dataSource={rows} columns={[{ title: '工单', dataIndex: 'id' }, { title: '分类', dataIndex: 'category' }, { title: '问题', dataIndex: 'description' }, { title: '状态', dataIndex: 'status' }, { title: '责任人', dataIndex: 'assigned_to' }, { title: '详情', render: (_, r) => <Button onClick={() => { closeImage(); select(r) }}>处理与历史</Button> }]} />
 {selected && <Card title={`工单 ${selected.id} · V${selected.version}`}><p>关联业务 {String(selected.related_id)}；处理队列 {String(selected.queue)}；结论 {String(selected.conclusion || '待核查')}</p>
 <Table<Row> rowKey={r => String(r.id)} dataSource={selected.replies as Row[]} columns={[{ title: '回复人', dataIndex: 'actor' }, { title: '可见范围', render: (_, r) => r.internal ? '仅内部' : '用户可见' }, { title: '内容', dataIndex: 'body' }, { title: '受控附件', render: (_, r) => (r.attachments as string[] || []).map(id => <Button key={id} onClick={() => void run(async () => { closeImage(); setImage(await viewPrivateImage(id, '处理当前工单核查附件')) })}>查看附件</Button>) }]} />
 {image && <div><img src={image} alt="当前授权工单附件" style={{ maxWidth: '100%' }} /><Button onClick={closeImage}>关闭附件</Button></div>}
 <Input.TextArea value={reply} onChange={e => setReply(e.target.value)} placeholder="回复或内部备注（不承诺未经核实的到账）" /><Select value={String(internal)} onChange={v => setInternal(v === 'true')} options={[{ value: 'false', label: '用户可见回复' }, { value: 'true', label: '仅内部备注' }]} /><Upload accept="image/png,image/jpeg" showUploadList={false} beforeUpload={file => { void run(async () => setAttachments([...attachments, await uploadSupportAttachment(file)])); return false }}><Button disabled={locked}>上传本次回复附件</Button></Upload><p>已选附件 {attachments.length} 张</p><Button disabled={locked || !reply} onClick={() => void submit('ticket_reply', { id: selected.id, version: selected.version, body: reply, internal, attachments, reason: '工单沟通回复' })}>发送回复</Button>
 <OperationForm locked={locked} label="转派／升级／退回并保留责任轨迹" fields={[{ name: 'assigned_to', label: '目标客服编号' }, { name: 'queue', label: '目标队列' }, { name: 'due_at', label: '处理截止时间', kind: 'datetime' }, { name: 'reason', label: '流转说明' }]} submit={v => submit('ticket_assign', { ...v, id: selected.id, version: selected.version })} />
 <OperationForm locked={locked} label="确认结案（不直接修复数字）" fields={[{ name: 'conclusion', label: '核查结论与下一步', kind: 'text' }, { name: 'adjustment_id', label: '正式数字调整单编号（可选）', optional: true }, { name: 'reason', label: '结案依据' }]} submit={v => submit('ticket_close', { ...v, id: selected.id, version: selected.version })} />
 </Card>}
 <Card title="显式配置申诉与重开政策（规则管理员）"><OperationForm locked={locked} label="发布工单政策" fields={[{ name: 'version', label: '当前政策版本（首次为0）', kind: 'integer' }, { name: 'appeal_seconds', label: '申诉期限（秒）', kind: 'integer' }, { name: 'max_appeals', label: '最多申诉次数', kind: 'integer' }, { name: 'reopen_seconds', label: '重开期限（秒）', kind: 'integer' }, { name: 'independent', label: '强制非原审核人独立复核', kind: 'boolean' }, { name: 'reason', label: '政策依据' }]} submit={v => submit('support_policy', { ...v, independent: v.independent === true })} /></Card>
 </Space>
}
