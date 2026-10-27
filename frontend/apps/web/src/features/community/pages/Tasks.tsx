import { tasks } from '../../tasks/api'
import { useEffect, useState } from 'react'
import { Alert, Button, Card, Checkbox, Descriptions, Empty, Input, Modal, Select, Space, Table, Tag, Typography } from 'antd'
import { taskStatusNames, viewEvidenceImage, type TaskInstance, type TaskPage, type TaskSubmission, type User } from '../api'
import type { Run, Save } from '../model'
export function Tasks({ me, busy, pending, run, save }: { me: User; busy: boolean; pending: boolean; run: Run; save: Save }) {
  const [data, setData] = useState<TaskPage>()
  const [status, setStatus] = useState<string>()
  const [selected, setSelected] = useState<TaskInstance>()
  const [reason, setReason] = useState('')
  const [code, setCode] = useState('')
  const [fields, setFields] = useState<string[]>([])
  const [checked, setChecked] = useState(false)
  const [evidence, setEvidence] = useState<TaskSubmission[]>()
  const [images, setImages] = useState<string[]>([])
  const [lookup, setLookup] = useState('')
  const [proposal, setProposal] = useState('')
  const [repairStatus, setRepairStatus] = useState('in_progress')
  const [proof, setProof] = useState('')
  const [batch, setBatch] = useState<TaskInstance[]>([])
  const [batchPreview, setBatchPreview] = useState<{ id: string; amount_cents: number; entries: { id: string; version: number }[] }>()
  const reviewer = me.roles.includes('task_reviewer')
  const locked = busy || pending
  async function load(cursor?: string) { const next = await tasks.queue(JSON.stringify({ status, cursor })); setData(old => cursor && old ? { ...next, items: [...old.items, ...next.items] } : next) }
  useEffect(() => { void run(() => load()) }, [status])
  useEffect(() => () => { images.forEach(url => URL.revokeObjectURL(url)) }, [images])
  async function detail(id: string) { setSelected(await tasks.detail(id)); setEvidence(undefined); setImages([]); setChecked(false); setReason(''); setFields([]) }
  async function act(action: string, extra: Record<string, unknown> = {}) { if (!selected) return; if (await save(action, { id: selected.id, version: selected.version, reason, ...extra })) { await run(async () => { await load(); await detail(selected.id) }) } }
  return <Space orientation="vertical" className="full-width">
    <Alert type="info" title="独占领取后再审核；补交不生成佣金，高风险初审须另一人员复核。超时仅提醒，不会自动通过。" />
    <Space wrap><Select aria-label="审核状态筛选" allowClear placeholder="全部待审状态" value={status} onChange={setStatus} options={['submitted', 'reviewing', 'awaiting_verification'].map(value => ({ value, label: taskStatusNames[value] }))} /><Button disabled={locked} onClick={() => void run(() => load())}>刷新待审任务</Button><Input aria-label="查询任务编号" placeholder="任务实例编号（含历史任务）" value={lookup} onChange={e => setLookup(e.target.value)} /><Button disabled={locked || !lookup} onClick={() => void run(() => detail(lookup))}>查询任务详情</Button></Space>
    <Table<TaskInstance> rowKey="id" dataSource={data?.items} rowSelection={{ selectedRowKeys: batch.map(t => t.id), onChange: (_, rows) => { setBatch(rows); setBatchPreview(undefined) } }} columns={[
      { title: '任务实例', dataIndex: 'id' }, { title: '任务类型', render: (_, t) => t.template.task_type }, { title: '状态', render: (_, t) => <Tag>{taskStatusNames[t.status] || t.status}</Tag> }, { title: '风险／等待', render: (_, t) => `${t.template.double_review ? '双人复核' : '常规'} · ${t.waiting_seconds || 0} 秒${t.overdue ? '（已超提醒时限）' : ''}` }, { title: '操作', render: (_, t) => <Button disabled={locked} onClick={() => void run(() => detail(t.id))}>处理详情</Button> },
    ]} locale={{ emptyText: <Empty description={data ? '暂无匹配待审任务' : '尚未加载任务'} /> }} />
    {data?.next_cursor && <Button disabled={locked} onClick={() => void run(() => load(data.next_cursor || undefined))}>加载更多</Button>}
    {reviewer && <Card title="批量通过（仅已独占领取的常规人工固定佣金任务）"><Space wrap><Input aria-label="批量审核原因" placeholder="批量核对说明" value={reason} onChange={e => setReason(e.target.value)} /><Button disabled={locked || !batch.length} onClick={() => void run(async () => { setBatchPreview(await tasks.reviewBatchPreview({ entries: batch.map(t => ({ id: t.id, version: t.version })) }, crypto.randomUUID())) })}>校验并预览选中任务</Button>{batchPreview && <><Typography.Text>共 {batchPreview.entries.length} 条，数字合计 {batchPreview.amount_cents} 分</Typography.Text><Button disabled={locked || !reason} onClick={async () => { if (await save('review_batch', { entries: batchPreview.entries, preview_id: batchPreview.id, reason, confirmed: true })) { setBatchPreview(undefined); setBatch([]); await run(() => load()) } }}>确认批量通过</Button></>}</Space></Card>}
    <Modal title="任务执行与质量审核" open={!!selected} onCancel={() => { setSelected(undefined); setEvidence(undefined); setImages([]) }} width={960} footer={null} destroyOnHidden>
      {selected && <Space orientation="vertical" className="full-width">
        <Descriptions bordered column={1} items={[
          { key: 'id', label: '实例／版本', children: `${selected.id} / ${selected.version}` }, { key: 'state', label: '状态', children: taskStatusNames[selected.status] || selected.status }, { key: 'completion', label: '领取时完成条件', children: selected.template.completion }, { key: 'failure', label: '失败条件', children: selected.template.failure_conditions }, { key: 'goal', label: '审核目标', children: selected.template.review_goal }, { key: 'price', label: '冻结佣金规则', children: JSON.stringify(selected.price) }, { key: 'receipt', label: '提交回执', children: selected.receipt ? `${selected.receipt.id} / 第 ${selected.receipt.version} 版 / ${new Date(selected.receipt.submitted_at * 1000).toLocaleString()}` : '尚未提交' },
        ]} />
        <Input.TextArea aria-label="处理原因" placeholder="处理原因／证据查看用途（必填）" value={reason} onChange={e => setReason(e.target.value)} />
        {reviewer && selected.status === 'submitted' && <Button type="primary" disabled={locked} onClick={() => void act('review_claim')}>独占领取审核</Button>}
        <Button disabled={locked || !reason} onClick={() => void run(async () => { const result = await tasks.evidenceReveal<{ submissions: TaskSubmission[] }>({ id: selected.id, version: selected.version, reason }, crypto.randomUUID()); setEvidence(result.submissions) })}>按用途授权查看提交证据（留痕）</Button>
        {evidence?.map(submission => <Card size="small" key={submission.id} title={`第 ${submission.version} 版 · ${submission.id}`}><Typography.Paragraph>发生时间：{submission.evidence.occurred_at ? new Date(submission.evidence.occurred_at * 1000).toLocaleString() : '未填写'}</Typography.Paragraph>{Object.entries(submission.evidence.fields).map(([key, value]) => <div key={key}><strong>{selected.template.fields.find(f => f.id === key)?.label || key}：</strong>{typeof value === 'string' ? value : value.map(id => <Button key={id} disabled={locked || !reason} onClick={() => void run(async () => { const url = await viewEvidenceImage(id, reason); setImages([url]) })}>查看私有图片</Button>)}</div>)}</Card>)}
        {images.map(url => <img key={url} src={url} alt="授权查看的任务证据" style={{ maxWidth: '100%', maxHeight: 420 }} />)}
        <Checkbox checked={checked} onChange={e => setChecked(e.target.checked)}>已核对领取快照、完成条件、全部必填证据、订单去重和定价版本</Checkbox>
        {selected.status === 'reviewing' && selected.reviewer === me.id && <>
          <Space wrap><Button type="primary" disabled={locked || !reason || !checked} onClick={() => void act('review_decide', { decision: 'approve', checklist_confirmed: checked })}>审核通过</Button><Input placeholder="驳回业务原因码" value={code} onChange={e => setCode(e.target.value)} /><Button danger disabled={locked || !reason || !code} onClick={() => void act('review_decide', { decision: 'reject', reason_code: code })}>最终驳回</Button></Space>
          <Select mode="multiple" className="full-width" placeholder="指定需要补交的证据字段" value={fields} onChange={setFields} options={selected.template.fields.map(f => ({ value: f.id, label: f.label }))} /><Button disabled={locked || !reason || !fields.length} onClick={() => void act('review_decide', { decision: 'supplement', fields })}>要求按原模板补交</Button>
        </>}
        {selected.status === 'awaiting_verification' && me.roles.includes('verifier') && <Space><Button type="primary" disabled={locked || !reason || !checked || selected.first_reviewer === me.id} onClick={() => void act('review_verify', { approve: true, checklist_confirmed: checked })}>独立复核通过</Button><Button disabled={locked || !reason || selected.first_reviewer === me.id} onClick={() => void act('review_verify', { approve: false })}>复核退回重审</Button></Space>}
        {me.roles.includes('operator') && <Space wrap><Button disabled={locked || !reason} onClick={() => void act('task_pause')}>暂停任务</Button><Button disabled={locked || !reason} onClick={() => void act('task_resume')}>恢复任务</Button><Button disabled={locked} onClick={() => void act('task_expire')}>按快照检查失效</Button></Space>}
        <Card size="small" title="更正与质量检查（保留原结论和历史）"><Input.TextArea placeholder="更正依据／关联证据说明" value={proof} onChange={e => setProof(e.target.value)} /><Select value={repairStatus} onChange={setRepairStatus} options={[{ value: 'claimed', label: '修复为已领取' }, { value: 'in_progress', label: '修复为执行中' }]} /><Space wrap>{me.roles.includes('operator') && <Button disabled={locked || !proof || !reason} onClick={() => void run(async () => { const p = await tasks.taskRepairPropose<{ id: string }>({ id: selected.id, version: selected.version, reason, evidence: proof, new_status: repairStatus }, crypto.randomUUID()); setProposal(p.id) })}>申请状态修复</Button>}<Button disabled={locked || !reviewer || !proof || !reason} onClick={() => void run(async () => { const p = await tasks.reviewRevokePropose<{ id: string }>({ id: selected.id, version: selected.version, reason, evidence: proof }, crypto.randomUUID()); setProposal(p.id) })}>申请撤销审核结论</Button><Button disabled={locked || !reviewer || !reason} onClick={() => void act('quality_inspect', { consistent: true })}>抽检一致</Button><Button disabled={locked || !reviewer || !reason} onClick={() => void act('quality_inspect', { consistent: false })}>抽检差异待更正</Button></Space><Select className="full-width" placeholder="从此任务已有更正申请恢复处理" value={proposal || undefined} onChange={setProposal} options={selected.corrections?.map(p => ({ value: p.id, label: `${p.id} · ${p.kind} · ${p.status} · ${p.reason}` }))} /><Input placeholder="更正申请编号" value={proposal} onChange={e => setProposal(e.target.value)} />{me.roles.includes('verifier') && <Space><Button disabled={locked || !reason || !proposal} onClick={() => void act('task_repair_review', { proposal_id: proposal, approve: true })}>独立批准状态修复</Button><Button disabled={locked || !reason || !proposal} onClick={() => void act('review_revoke_review', { proposal_id: proposal, approve: true })}>独立批准结论转争议</Button></Space>}<Typography.Paragraph type="secondary">结论转争议只追加数字暂缓核查事件；不会删除原记录、追回外部款项或自动抵扣。</Typography.Paragraph></Card>
        <Card size="small" title="审核历史">{selected.decisions.map((d, i) => <p key={i}>{d.actor} · {d.decision} · {d.reason}</p>)}</Card>
      </Space>}
    </Modal>
  </Space>
}
