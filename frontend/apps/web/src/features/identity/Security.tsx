import { useState } from 'react'
import { Alert, Button, Card, Input, Space, Table, Tabs } from 'antd'
import { identity } from './api'
import { OperationForm } from '../../shared/OperationForm'
import type { Run, Save } from '../platform/model'
export function Security({ locked, run, save }: { locked: boolean; run: Run; save: Save }) {
 const [policy, setPolicy] = useState<Record<string, unknown> | null>(null)
 const [grants, setGrants] = useState<Record<string, unknown>[]>([])
 const [password, setPassword] = useState('')
 const load = () => run(async () => { const v = await identity.security(); setPolicy(v.policy); setGrants(v.grants) })
 const submit = (action: string) => async (v: Record<string, unknown>) => { if (await save(action, v)) await load() }
 return <Space orientation="vertical" className="full-width"><Alert type="info" title="临时授权按角色与对象范围逐次校验，审批与申请分离，到期立即拒绝新操作。再次验证只绑定当前会话。" /><Card title="当前会话再次验证"><Space><Input.Password value={password} autoComplete="current-password" aria-label="后台密码再次验证" onChange={e => setPassword(e.target.value)} /><Button disabled={locked || !password} onClick={() => void run(async () => { try { await identity.reauthenticate(password) } finally { setPassword('') } })}>验证当前会话</Button></Space></Card><Button disabled={locked} onClick={() => void load()}>刷新安全政策与临时授权</Button><Tabs items={[
 { key: 'policy', label: '高风险验证政策', children: <Card><OperationForm key={String(policy?.version || 0)} locked={locked} label="发布政策" initial={{ version: policy?.version || 0 }} fields={[{ name: 'version', label: '当前版本', kind: 'integer' }, { name: 'reauth_seconds', label: '再次验证有效秒数', kind: 'integer' }, { name: 'actions', label: '需再次验证的动作', kind: 'tags' }, { name: 'reason', label: '发布依据', kind: 'text' }]} submit={submit('security_policy_publish')} /><pre>{JSON.stringify(policy, null, 2)}</pre></Card> },
 { key: 'grant', label: '应急授权申请', children: <OperationForm locked={locked} label="提交独立审批" fields={[{ name: 'user_id', label: '成员编号' }, { name: 'roles', label: '临时角色', kind: 'tags' }, { name: 'scope', label: '明确数据范围', kind: 'tags' }, { name: 'expires_at', label: '到期时间', kind: 'datetime' }, { name: 'reason', label: '应急依据', kind: 'text' }]} submit={submit('temporary_grant_request')} /> },
 { key: 'review', label: '审批与撤回', children: <><Table rowKey={v => String(v.id)} dataSource={grants} columns={[{ title: '编号', dataIndex: 'id' }, { title: '成员', dataIndex: 'user_id' }, { title: '状态', dataIndex: 'status' }, { title: '版本', dataIndex: 'version' }, { title: '到期', render: (_, v) => new Date(Number(v.expires_at) * 1000).toLocaleString() }]} /><OperationForm locked={locked} label="独立审批" fields={[{ name: 'id', label: '授权编号' }, { name: 'version', label: '当前版本', kind: 'integer' }, { name: 'approve', label: '批准', kind: 'boolean' }, { name: 'reason', label: '审批依据', kind: 'text' }]} submit={submit('temporary_grant_review')} /><OperationForm locked={locked} label="立即撤回临时授权" fields={[{ name: 'id', label: '授权编号' }, { name: 'version', label: '当前版本', kind: 'integer' }, { name: 'reason', label: '撤回依据' }]} submit={submit('temporary_grant_revoke')} /></> },
 { key: 'source', label: '任务成果来源', children: <OperationForm locked={locked} label="更新任务来源启停" fields={[{ name: 'source', label: '来源编号（验签密钥由部署单独配置）' }, { name: 'version', label: '当前来源配置版本，首次为0', kind: 'integer' }, { name: 'enabled', label: '允许接收此来源', kind: 'boolean' }, { name: 'reason', label: '变更依据' }]} submit={submit('task_source_configure')} /> },
 ]} /></Space>
}
