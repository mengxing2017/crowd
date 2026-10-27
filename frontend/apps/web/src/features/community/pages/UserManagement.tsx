import { useState } from 'react'
import { Alert, Button, Card, Descriptions, Form, Input, Select, Space, Tag, Tabs, Typography } from 'antd'
import { SearchOutlined } from '@ant-design/icons'
import { community } from '../api'
import { roleNames, statuses, type Run, type Save } from '../model'
interface ManagedUser { id: string; status: string; version: number; roles: string[]; scope: string[]; restrictions: { scope: string; reason: string; appeal: string }[] }
export function UserManagement({ actorId, busy, pending, run, save }: { actorId: string; busy: boolean; pending: boolean; run: Run; save: Save }) {
  const [eligibility, setEligibility] = useState<{ version: number; region?: string; qualifications?: string[] }>({ version: 0 })
  const [eligibilityForm] = Form.useForm()
  const [user, setUser] = useState<ManagedUser>()
  const [accessForm] = Form.useForm()
  const [restrictionForm] = Form.useForm()
  async function load(id: string) {
    setUser(undefined)
    const next = await community.read<ManagedUser>('users', id)
    const verified = await community.read<{ version: number; region?: string; qualifications?: string[] }>('eligibility', id)
    setEligibility(verified); eligibilityForm.setFieldsValue({ region: verified.region, qualifications: verified.qualifications || [], reason: '' })
    setUser(next); accessForm.setFieldsValue({ roles: next.roles, scope: next.scope, reason: '' }); restrictionForm.resetFields()
  }
  const locked = busy || pending || user?.id === actorId
  return <Space orientation="vertical" size={24} className="full-width">
    <Card title="查询用户"><Form layout="inline" onFinish={values => { void run(() => load(String(values.target).trim())) }}><Form.Item name="target" label="用户编号" rules={[{ required: true, whitespace: true, message: '请输入用户编号' }]}><Input placeholder="输入完整用户编号" allowClear style={{ width: 320, maxWidth: '100%' }} /></Form.Item><Form.Item><Button type="primary" htmlType="submit" icon={<SearchOutlined />} loading={busy}>查询</Button></Form.Item></Form></Card>
    {!user ? <Card><Typography.Paragraph type="secondary">输入用户编号，查询其状态、岗位与已授权的数据范围。查询执行服务端对象范围检查。</Typography.Paragraph></Card> : <>
      <Card title="用户档案"><Descriptions column={{ xs: 1, md: 2 }} items={[{ key: 'id', label: '用户编号', children: <Typography.Text copyable>{user.id}</Typography.Text> }, { key: 'status', label: '账号状态', children: <Tag color={user.status === 'active' ? 'success' : 'error'}>{statuses[user.status] || user.status}</Tag> }, { key: 'roles', label: '岗位', children: <Space wrap>{user.roles.map(role => <Tag key={role}>{roleNames[role] || role}</Tag>)}</Space> }, { key: 'scope', label: '数据范围', children: user.scope.includes('*') ? '全部已授权对象' : user.scope.join('、') || '暂无' }, { key: 'version', label: '资料版本', children: `V${user.version}` }]} />{user.restrictions.length > 0 && <Alert type="warning" showIcon title="该账号存在业务限制" description={user.restrictions.map(item => `${item.scope === 'all' ? '全部业务' : item.scope === 'promotion' ? '推广' : '申请'}：${item.reason}；${item.appeal}`).join('；')} />}</Card>
      <Card>{user.id === actorId && <Alert type="info" showIcon title="不能修改自己的权限或限制" />}<Tabs items={[
        { key: 'access', label: '岗位与数据范围', children: <Form form={accessForm} layout="vertical" disabled={locked} onFinish={async values => { const id = user.id; if (await save('access', { ...values, user_id: id, version: user.version })) await run(() => load(id)) }}><Form.Item name="roles" label="授权岗位" rules={[{ required: true, message: '请选择岗位' }]}><Select mode="multiple" options={Object.entries(roleNames).map(([value, label]) => ({ value, label }))} /></Form.Item><Form.Item name="scope" label="数据范围" extra="填写可访问的用户编号；platform 用于平台规则；* 表示全部已授权对象。" rules={[{ required: true, message: '请填写数据范围' }]}><Select mode="tags" tokenSeparators={[',', '，']} placeholder="输入范围后按回车" /></Form.Item><Form.Item name="reason" label="变更原因" rules={[{ required: true, whitespace: true, message: '请填写变更原因' }]}><Input.TextArea rows={3} /></Form.Item><Button type="primary" htmlType="submit" loading={busy}>保存岗位与范围</Button></Form> },
        { key: 'restriction', label: '设置业务限制', children: <Form form={restrictionForm} layout="vertical" disabled={locked} onFinish={async values => { const id = user.id; if (await save('restriction', { ...values, user_id: id, version: user.version })) await run(() => load(id)) }}><Alert type="warning" showIcon title="限制操作将保留记录" description="限制全部业务会停用账号；推广或申请限制只约束对应业务。" /><Form.Item name="restriction_scope" label="限制范围" rules={[{ required: true, message: '请选择限制范围' }]}><Select options={[{ value: 'promotion', label: '推广业务' }, { value: 'withdrawal', label: '申请业务' }, { value: 'all', label: '全部业务（停用账号）' }]} /></Form.Item><Form.Item name="reason" label="限制原因" rules={[{ required: true, whitespace: true, message: '请填写限制原因' }]}><Input.TextArea rows={3} /></Form.Item><Button danger type="primary" htmlType="submit" loading={busy}>确认设置限制</Button></Form> },
        { key: 'eligibility', label: '地区与资格核验', children: <Form form={eligibilityForm} layout="vertical" disabled={busy || pending} onFinish={async values => { if (await save('eligibility_verify', { ...values, user_id: user.id, version: eligibility.version })) await run(() => load(user.id)) }}><Alert type="info" title="根据业务依据人工核验后录入。领取不能使用客户端自报地区或资格。每次修改保留历史。" /><Form.Item name="region" label="地区代码" rules={[{ required: true }]}><Input /></Form.Item><Form.Item name="qualifications" label="已核验资格"><Select mode="tags" /></Form.Item><Form.Item name="reason" label="核验依据" rules={[{ required: true }]}><Input.TextArea /></Form.Item><Button htmlType="submit" type="primary">保存核验记录</Button></Form> },
      ]} /></Card>
    </>}
  </Space>
}
