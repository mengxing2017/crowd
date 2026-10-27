import { identity } from '../identity/api'
import { useState } from 'react'
import { LockOutlined, SafetyCertificateOutlined, UserOutlined } from '@ant-design/icons'
import { LoginForm, ProFormText } from '@ant-design/pro-components'
import { Alert, Form, Tabs, Typography } from 'antd'
import { setToken, type Login } from './api'
interface Credentials { username: string; password: string; ticket: string }
export function AdminLogin({ onAuthenticated, initialError }: { onAuthenticated: (session: Login) => Promise<void>; initialError?: string }) {
  const [mode, setMode] = useState('password')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState(initialError || '')
  const [form] = Form.useForm<Credentials>()
  return <div className="login-page"><header className="login-header"><span className="brand-mark">C</span><span>Crowd</span><span className="login-header-label">运营管理平台</span></header><main className="login-content">
    <LoginForm<Credentials> form={form} title="Crowd 运营管理" subTitle="让业务有据可依，让每一次协作都有记录" logo={<SafetyCertificateOutlined style={{ color: '#1677ff', fontSize: 36 }} />} initialValues={{ username: 'admin' }} submitter={{ searchConfig: { submitText: busy ? '正在登录…' : '登录' }, submitButtonProps: { 'aria-label': '登录', loading: busy, size: 'large', block: true } }} onFinish={async values => {
      if (busy) return false
      setBusy(true); setError('')
      try { const session = mode === 'password' ? await identity.adminLogin({ username: values.username, password: values.password }) : await identity.redeem(values.ticket); await onAuthenticated(session); form.resetFields(['password', 'ticket']); return true }
      catch (error) { setToken(''); setError(error instanceof Error ? error.message : '登录失败，请稍后重试'); return false }
      finally { setBusy(false) }
    }}>
      <Tabs activeKey={mode} centered items={[{ key: 'password', label: '账号密码登录', disabled: busy }, { key: 'ticket', label: '小程序票据登录', disabled: busy }]} onChange={value => { setMode(value); setError(''); form.resetFields(['password', 'ticket']) }} />
      {error && <Alert className="login-alert" showIcon type="error" title={error} />}
      {mode === 'password' ? <><ProFormText name="username" label="管理员账号" disabled={busy} fieldProps={{ size: 'large', prefix: <UserOutlined />, autoComplete: 'username', maxLength: 64 }} placeholder="请输入管理员账号" rules={[{ required: true, whitespace: true, message: '请输入管理员账号' }]} /><ProFormText.Password name="password" label="密码" disabled={busy} fieldProps={{ size: 'large', prefix: <LockOutlined />, autoComplete: 'current-password', maxLength: 128 }} placeholder="请输入密码" rules={[{ required: true, message: '请输入密码' }]} /></> : <><Typography.Paragraph type="secondary">在已登录的小程序个人中心生成一次性票据，有效期 60 秒。</Typography.Paragraph><ProFormText name="ticket" label="一次性登录票据" disabled={busy} fieldProps={{ size: 'large', autoComplete: 'off' }} placeholder="请输入一次性登录票据" rules={[{ required: true, whitespace: true, message: '请输入一次性票据' }]} /></>}
    </LoginForm><Typography.Paragraph type="secondary" className="login-caption">仅限获授权的工作人员使用 · 按岗位与数据范围授权</Typography.Paragraph>
  </main><footer className="login-footer">Crowd · 产品推广与协作平台<p>佣金仅为业务数字，实际付款由人员在线下完成。</p></footer></div>
}
