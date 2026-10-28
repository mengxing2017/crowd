import { tasks } from './api'
import { useEffect, useState } from 'react'
import { Alert, Button, Card, Collapse, Form, Input, InputNumber, Modal, Select, Space, Switch, Table, Tabs, Typography } from 'antd'
import { type TaskRules as Rules, type VersionedTaskRule, type PriceRule, type TaskTemplate } from './api'
import { type User } from '../identity/api'
import type { Run, Save } from '../platform/model'
const required = [{ required: true, message: '请明确填写此项' }]
const kinds = (values: string[]) => values.map(value => ({ value, label: value }))
const split = (v?: string) => (v || '').split(/[，,\n]/).map(s => s.trim()).filter(Boolean)
const date = (v: string) => Math.floor(new Date(v).getTime() / 1000)
export function TaskRules({ me, busy, pending, run, save }: { me: User; busy: boolean; pending: boolean; run: Run; save: Save }) {
  const [data, setData] = useState<Rules>()
  const [editor, setEditor] = useState<'price' | 'template'>()
  const [form] = Form.useForm()
  const [review, setReview] = useState<{ kind: 'price' | 'template'; id: string; version: number; editor: string }>()
  const [reason, setReason] = useState('')
  const locked = busy || pending
  async function load() { setData(await tasks.rules()) }
  useEffect(() => { void run(load) }, [])
  async function submit(action: string, body: Record<string, unknown>) { if (await save(action, body)) { await run(load); return true } return false }
  function open(kind: 'price' | 'template') { form.resetFields(); setEditor(kind) }
  function rules(kind: 'price' | 'template') {
    return <Table<VersionedTaskRule<PriceRule | TaskTemplate>> rowKey="id" dataSource={kind === 'price' ? data?.prices : data?.templates} columns={[
      { title: '版本编号', dataIndex: 'id' }, { title: '状态', dataIndex: 'status' }, { title: '编制人员', dataIndex: 'editor' },
      { title: '内容', render: (_, row) => <Collapse items={[{ key: 'content', label: '查看固定规则与条件', children: <Typography.Paragraph style={{ whiteSpace: 'pre-wrap' }}>{JSON.stringify(row.content, null, 2)}</Typography.Paragraph> }]} /> },
      { title: '操作', render: (_, row) => <Space>{row.status === 'pending' && me.roles.includes('verifier') && <Button disabled={locked || row.editor === me.id} onClick={() => { setReason(''); setReview({ kind, ...row }) }}>独立审核</Button>}{kind === 'template' && row.status === 'approved' && me.roles.includes('operator') && <Button disabled={locked || !reason} onClick={() => void submit('template_disable', { id: row.id, version: row.version, reason })}>停止新领取</Button>}</Space> },
    ]} />
  }
  return <Space orientation="vertical" className="full-width">
    <Alert type="info" showIcon title="版本通过独立审核后才能关联产品。修改须创建新编号；已领取任务始终使用原快照。" description="佣金为业务标记数字。比例佣金显示计算方式，可选封顶，实际数字由可信成果基数计算；审核后仍需后续数字入账与人工线下结算。" />
    <Space>{me.roles.includes('rule_manager') && <Button disabled={locked} onClick={() => open('price')}>新建定价版本</Button>}{me.roles.includes('operator') && <Button disabled={locked} onClick={() => open('template')}>新建任务模板</Button>}<Input aria-label="模板停用原因" placeholder="模板停用原因" value={reason} onChange={e => setReason(e.target.value)} /><Button disabled={locked} onClick={() => void run(load)}>刷新规则</Button></Space>
    <Tabs items={[{ key: 'prices', label: '佣金定价', children: rules('price') }, { key: 'templates', label: '任务模板与资格', children: rules('template') }]} />
    <Modal title={editor === 'price' ? '新建佣金定价版本' : '新建任务模板版本'} open={!!editor} onCancel={() => setEditor(undefined)} footer={null} width={860} destroyOnHidden>
      <Form form={form} layout="vertical" disabled={locked} onFinish={async values => {
        const { id, start, end, claim_end, regions_text, qualifications_text, ...content } = values
        if (editor === 'price') { content.starts_at = date(start); content.ends_at = date(end); content.currency = 'CNY'; if (content.period_cap_cents !== undefined && content.period_cap_cents !== null) content.reservation_policy = 'reserve_max_release_terminal' }
        else { content.claim_until = date(claim_end); content.regions = split(regions_text); content.qualifications = split(qualifications_text); for (const k of ['allow_abandon', 'allow_withdraw', 'release_on_terminal', 'double_review']) content[k] = content[k] === true }
        if (await submit(`${editor}_save`, { id, version: 0, content })) setEditor(undefined)
      }}>
        <Form.Item name="id" label="新版本唯一编号" rules={required}><Input /></Form.Item>
        {editor === 'price' ? <>
          {['task_id', 'settlement_scope'].map((name, i) => <Form.Item key={name} name={name} label={['适用任务编号', '结算范围编号'][i]} rules={required}><Input /></Form.Item>)}
          <Form.Item name="kind" label="定价方式" rules={required}><Select options={[{ value: 'fixed', label: '固定整数分' }, { value: 'proportional', label: '可信基数比例（可选封顶）' }]} /></Form.Item>
          <Form.Item name="amount_cents" label="固定数字（整数分）"><InputNumber min={0} precision={0} /></Form.Item>
          <Card size="small" title="比例方式须填写以下条件"><Space wrap>{['numerator', 'denominator', 'cap_cents'].map((name, i) => <Form.Item key={name} name={name} label={['比例分子', '比例分母（明确精度）', '可选单笔封顶（分）'][i]}><InputNumber min={0} precision={0} /></Form.Item>)}</Space><Form.Item name="rounding" label="舍入"><Select options={kinds(['floor', 'half_up', 'ceil'])} /></Form.Item><Form.Item name="base_source" label="可信成果基数来源编号"><Input /></Form.Item></Card>
          <Form.Item name="start" label="生效时间" rules={required}><Input type="datetime-local" /></Form.Item><Form.Item name="end" label="失效时间" rules={required}><Input type="datetime-local" /></Form.Item>
          <Form.Item name="group" label="适用经核验资格标签（可选）"><Input /></Form.Item>
          <Card size="small" title="可选用户周期上限（占用最大单笔数字，终止按策略释放）">{['period_cap_cents', 'period_seconds', 'period_anchor'].map((name, i) => <Form.Item key={name} name={name} label={['周期上限（分）', '周期长度（秒）', '周期起点（UTC 秒）'][i]}><InputNumber min={0} precision={0} /></Form.Item>)}</Card>
        </> : <>
          {['name', 'task_type', 'completion', 'failure_conditions', 'review_goal'].map((name, i) => <Form.Item key={name} name={name} label={['模板名称', '具体业务任务类型', '完成条件', '失效／失败条件', '审核目标'][i]} rules={required}><Input.TextArea /></Form.Item>)}
          <Form.Item name="source" label="成果来源" rules={required}><Select options={[{ value: 'manual', label: '人工审核证据' }, { value: 'external', label: '可信外部成果与人工判定' }, { value: 'both', label: '证据与可信外部成果同时核对' }]} /></Form.Item>
          <Form.Item name="external_source" label="可信外部成果来源编号（外部／双来源必填）"><Input /></Form.Item>
          <Form.Item name="claim_end" label="停止领取时间" rules={required}><Input type="datetime-local" /></Form.Item>
          {['execution_seconds', 'submit_seconds', 'total_quota', 'per_user_quota', 'review_sla_seconds', 'max_evidence_files', 'max_file_bytes', 'max_supplements', 'supplement_seconds'].map((name, i) => <Form.Item key={name} name={name} label={['领取后执行时长（秒）', '领取后提交时长（秒）', '模板总名额', '个人名额', '审核提醒时限（秒）', '每份证据图片数（最多20）', '每张图片最大字节（最多5242880）', '最多补交次数', '每次补交时长（秒）'][i]} rules={required}><InputNumber min={0} precision={0} /></Form.Item>)}
          <Form.Item name="participation" label="参与频次" rules={required}><Select options={[{ value: 'once', label: '仅一次' }, { value: 'period', label: '按明确周期' }, { value: 'repeat', label: '允许重复（受个人名额约束）' }]} /></Form.Item>
          <Form.Item name="period_seconds" label="频次周期长度（秒）"><InputNumber min={1} precision={0} /></Form.Item><Form.Item name="period_anchor" label="频次周期起点（UTC 秒）"><InputNumber precision={0} /></Form.Item>
          <Form.Item name="regions_text" label="已核验地区（逗号分隔）"><Input /></Form.Item><Form.Item name="qualifications_text" label="已核验资格（逗号分隔）"><Input /></Form.Item>
          <Form.Item name="pause_policy" label="暂停截止策略" rules={required}><Select options={[{ value: 'extend', label: '按实际暂停时长顺延' }, { value: 'keep_deadline', label: '维持原截止时间' }]} /></Form.Item>
          {['allow_abandon', 'release_on_terminal', 'allow_withdraw', 'double_review'].map((name, i) => <Form.Item key={name} name={name} label={['执行前／执行中允许放弃', '放弃／失效／驳回释放计划名额及数字', '未被审核领取前允许撤回', '高风险须独立双人复核'][i]} valuePropName="checked"><Switch /></Form.Item>)}
          <Form.List name="steps" rules={[{ validator: async (_, value) => { if (!value?.length) throw new Error('至少添加一个步骤') } }]}>{(fields, { add, remove }, { errors }) => <Card title="按顺序执行步骤" extra={<Button onClick={() => add({ required: true })}>添加步骤</Button>}>{fields.map(f => <Space key={f.key} align="baseline"><Form.Item name={[f.name, 'id']} label="步骤编号" rules={required}><Input /></Form.Item><Form.Item name={[f.name, 'title']} label="操作说明" rules={required}><Input /></Form.Item><Form.Item name={[f.name, 'required']} label="必须完成" valuePropName="checked"><Switch /></Form.Item><Button onClick={() => remove(f.name)}>移除</Button></Space>)}<Form.ErrorList errors={errors} /></Card>}</Form.List>
          <Form.List name="fields">{(fields, { add, remove }) => <Card title="证据字段" extra={<Button onClick={() => add({ required: true })}>添加证据字段</Button>}>{fields.map(f => <Card key={f.key} size="small" extra={<Button onClick={() => remove(f.name)}>移除字段</Button>}>
            {['id', 'label', 'purpose'].map((name, i) => <Form.Item key={name} name={[f.name, name]} label={['字段编号', '字段标签', '收集用途'][i]} rules={required}><Input /></Form.Item>)}
            <Form.Item name={[f.name, 'kind']} label="证据类型" rules={required}><Select options={[{ value: 'text', label: '文本' }, { value: 'image', label: '私有图片' }, { value: 'order', label: '订单编号' }]} /></Form.Item><Form.Item name={[f.name, 'required']} label="必填" valuePropName="checked"><Switch /></Form.Item><Form.Item name={[f.name, 'max_length']} label="文本最大字数"><InputNumber min={1} precision={0} /></Form.Item><Form.Item name={[f.name, 'source']} label="订单来源编号"><Input /></Form.Item><Form.Item name={[f.name, 'exclusive_scope']} label="订单互斥业务范围"><Input /></Form.Item>
          </Card>)}</Card>}</Form.List>
        </>}
        <Button type="primary" htmlType="submit">保存并等待另一人员审核</Button>
      </Form>
    </Modal>
    <Modal title="独立规则审核" open={!!review} onCancel={() => setReview(undefined)} footer={<Space><Button disabled={locked || !reason} onClick={async () => { if (review && await submit(`${review.kind}_review`, { id: review.id, version: review.version, approve: false, reason })) setReview(undefined) }}>驳回</Button><Button type="primary" disabled={locked || !reason} onClick={async () => { if (review && await submit(`${review.kind}_review`, { id: review.id, version: review.version, approve: true, reason })) setReview(undefined) }}>批准此版本</Button></Space>}><Input.TextArea aria-label="规则审核原因" placeholder="核对条件、定价、时间和适用范围后的结论" value={reason} onChange={e => setReason(e.target.value)} /></Modal>
  </Space>
}
