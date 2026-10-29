import { useState } from 'react'
import { Alert, Button, Card, Descriptions, Form, Input, InputNumber, Modal, Select, Space, Table, Tag } from 'antd'
import { commissions, type CommissionSource, type CommissionSummary, type CommissionEntry, type CommissionProposal, type CommissionDispute } from './api'
import type { User } from '../identity/api'
import type { Run, Save } from '../platform/model'
const required = [{ required: true, message: '请填写此项' }]
const names: Record<string, string> = { pending: '待审记录', confirmed: '审核确认', increase: '补记增加', decrease: '关联调减', hold: '申请占用', release: '申请释放', settled: '人工结算登记', dispute_hold: '争议暂缓', dispute_release: '解除暂缓' }
export function Commissions({ me, busy, pending, run, save }: { me: User; busy: boolean; pending: boolean; run: Run; save: Save }) {
 const [owner, setOwner] = useState('')
 const [summary, setSummary] = useState<CommissionSummary>()
 const [entries, setEntries] = useState<CommissionEntry[]>([])
 const [next, setNext] = useState<number | null>(null)
 const [proposals, setProposals] = useState<CommissionProposal[]>([])
 const [disputes, setDisputes] = useState<CommissionDispute[]>([])
 const [reason, setReason] = useState('')
 const [source, setSource] = useState<CommissionSource>()
 const locked = busy || pending
 async function load(cursor = 0) { if (!owner.trim()) throw new Error('请指定授权范围内的用户'); const [s, p, a, d] = await Promise.all([commissions.summary(owner), commissions.entries(`${owner}|${cursor}`), commissions.proposals(owner), commissions.disputes(owner)]); setSummary(s); setEntries(cursor ? old => [...old, ...p.items] : p.items); setNext(p.next_cursor); setProposals(a); setDisputes(d) }
 async function submit(action: string, body: Record<string, unknown>) { if (await save(action, body)) await run(() => load()) }
 return <Space orientation="vertical" className="full-width">
  <Alert type="info" showIcon title="佣金仅为平台数字记录，非存款余额；收款及付款由人员在线下处理。" />
  <Space><Input aria-label="佣金用户编号" placeholder="用户编号" value={owner} onChange={e => { setOwner(e.target.value); setSummary(undefined); setEntries([]); setProposals([]); setDisputes([]) }} /><Button disabled={locked} onClick={() => void run(() => load())}>查询数字及流水</Button></Space>
  {summary && <><Alert type={summary.consistent ? 'success' : 'error'} title={summary.consistent ? '流水与申请占用一致' : '数字异常，暂停新增申请并核查'} /><Descriptions bordered items={['gross_confirmed', 'adjustment', 'pending', 'available', 'held', 'disputed', 'settled'].map((key, index) => ({ key, label: ['累计确认（分）', '净调整（分）', '待审（分）', '可申请（分）', '申请占用（分）', '争议暂缓（分）', '已登记结算（分）'][index], children: String(summary[`${key}_cents` as keyof CommissionSummary] ?? '当前服务未提供') }))} />
  <Table<CommissionEntry> rowKey="id" dataSource={entries} pagination={false} columns={[{ title: '业务键', dataIndex: 'id' }, { title: '变更', dataIndex: 'kind', render: kind => names[kind] || kind }, { title: '标记金额（分）', dataIndex: 'amount_cents' }, { title: '来源编号', render: (_, e) => <Button type="link" disabled={locked} onClick={() => void run(async () => setSource(await commissions.source(owner, e.source)))}>{e.source}</Button> }, { title: '操作人', dataIndex: 'actor' }, { title: '计算依据', render: (_, e) => <details><summary>历史规则及输入</summary><pre>{JSON.stringify(e.detail, null, 2)}</pre></details> }]} />{next !== null && <Button disabled={locked} onClick={() => void run(() => load(next))}>加载更多流水</Button>}
  {me.roles.includes('settlement_reviewer') && <Card title="补记或受益人纠错（须独立复核）"><Form layout="vertical" onFinish={v => void submit('commission_adjust_propose', { ...v, owner })}>
   {['business_key', 'source', 'reason', 'new_owner'].map((key, index) => <Form.Item key={key} name={key} label={['唯一业务单号', '任务或业务依据编号', '依据与原因', '新受益人（仅归属纠错，选填）'][index]} rules={key === 'new_owner' ? [] : required}><Input /></Form.Item>)}
   <Form.Item name="amount_cents" label="变更整数分（调减填负数）" rules={required}><InputNumber precision={0} /></Form.Item><Button htmlType="submit" disabled={locked}>提交调整复核</Button>
  </Form></Card>}
  <Input aria-label="数字核查说明" placeholder="复核／争议处理说明" value={reason} onChange={e => setReason(e.target.value)} />
  <Table<CommissionProposal> rowKey="id" dataSource={proposals} columns={[{ title: '调整单', dataIndex: 'id' }, { title: '标记变更（分）', dataIndex: 'amount_cents' }, { title: '依据', dataIndex: 'reason' }, { title: '状态', dataIndex: 'status' }, { title: '独立复核', render: (_, p) => p.status === 'pending' && p.actor !== me.id && me.roles.includes('verifier') && <Space>{[true, false].map(approve => <Button key={String(approve)} disabled={locked || !reason.trim()} onClick={() => void submit('commission_adjust_review', { id: p.id, approve, reason })}>{approve ? '批准调整' : '驳回调整'}</Button>)}</Space> }]} />
  {me.roles.includes('settlement_reviewer') && <Card title="对应数字争议暂缓"><Form layout="inline" onFinish={v => void submit('commission_dispute', { ...v, owner })}><Form.Item name="source" label="争议来源" rules={required}><Input /></Form.Item><Form.Item name="amount_cents" label="整数分" rules={required}><InputNumber min={1} precision={0} /></Form.Item><Form.Item name="reason" label="原因" rules={required}><Input /></Form.Item><Button htmlType="submit" disabled={locked}>暂缓对应数字</Button></Form></Card>}
  <Table<CommissionDispute> rowKey="id" dataSource={disputes} columns={[{ title: '争议', dataIndex: 'id' }, { title: '范围（分）', dataIndex: 'amount_cents' }, { title: '原因', dataIndex: 'reason' }, { title: '进度', render: (_, d) => <Tag>{d.status}</Tag> }, { title: '处理', render: (_, d) => d.status === 'open' && me.roles.includes('verifier') && <Button disabled={locked || !reason.trim()} onClick={() => void submit('commission_dispute_resolve', { id: d.id, reason, confirmed: true })}>确认核查并解除暂缓</Button> }]} />
  <Select aria-label="流水说明" value="platform" options={[{ value: 'platform', label: '增加、占用、释放及结算均可追溯来源；没有充值或资金转入' }]} />
  <Modal open={!!source} title="数字来源及关联申请" onCancel={() => setSource(undefined)} footer={null} width={800}>{source && <><p>来源：{source.source}；任务状态：{source.task?.status || '关联业务记录'}</p><Table rowKey="id" dataSource={source.applications} columns={[{ title: '申请', dataIndex: 'id' }, { title: '状态', dataIndex: 'status' }, { title: '标记金额（分）', dataIndex: 'amount_cents' }, { title: '原规则版本', dataIndex: 'rule_version' }]} /><pre>{JSON.stringify(source.task?.calculation || source.records.map(e => e.detail), null, 2)}</pre></>}</Modal>
  </>}
 </Space>
}
