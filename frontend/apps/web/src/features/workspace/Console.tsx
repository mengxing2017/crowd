import { executeCommand, platform } from '../platform/api'
import { identity } from '../identity/api'
import { governance } from '../governance/api'
import { lazy, Suspense, useEffect, useRef, useState } from 'react'
import { BookOutlined, DashboardOutlined, FileProtectOutlined, LogoutOutlined, QrcodeOutlined, ReloadOutlined, SafetyCertificateOutlined, TeamOutlined, UserOutlined } from '@ant-design/icons'
import { PageContainer, ProLayout } from '@ant-design/pro-components'
import { Alert, Button, Dropdown, Space, Spin, Tag, Tooltip, Result, Modal, Input } from 'antd'
import { AdminLogin } from '../identity/AdminLogin'
import { setToken, hasToken } from '../session/api'
import { BusinessError } from '../platform/api'
import { type Rule } from '../governance/api'
import { type User } from '../identity/api'
import { sections, type Section } from './model'
const Welcome = lazy(() => import('./pages/Welcome').then(module => ({ default: module.Welcome })))
const RuleCenter = lazy(() => import('../governance/RuleCenter').then(module => ({ default: module.RuleCenter })))
const UserManagement = lazy(() => import('../identity/UserManagement').then(module => ({ default: module.UserManagement })))
const Promotions = lazy(() => import('../promotion/Promotions').then(module => ({ default: module.Promotions })))
const AuditLog = lazy(() => import('../audit/AuditLog').then(module => ({ default: module.AuditLog })))
const Guide = lazy(() => import('./pages/Guide').then(module => ({ default: module.Guide })))
const Catalog = lazy(() => import('../catalog/Catalog').then(module => ({ default: module.Catalog })))
const Attribution = lazy(() => import('../attribution/Attribution').then(module => ({ default: module.Attribution })))
const TaskRules = lazy(() => import('../tasks/TaskRules').then(module => ({ default: module.TaskRules })))
const Tasks = lazy(() => import('../tasks/Tasks').then(module => ({ default: module.Tasks })))
const Commissions = lazy(() => import('../commissions/Commissions').then(m => ({ default: m.Commissions })))
const WithdrawalRules = lazy(() => import('../withdrawals/WithdrawalRules').then(m => ({ default: m.WithdrawalRules })))
const Settlements = lazy(() => import('../settlements/Settlements').then(m => ({ default: m.Settlements })))
const Campaigns = lazy(() => import('../campaigns/Campaigns').then(m => ({ default: m.Campaigns })))
const Adjustments = lazy(() => import('../commissions/Adjustments').then(m => ({ default: m.Adjustments })))
const Support = lazy(() => import('../support/Support').then(m => ({ default: m.Support })))
const Risk = lazy(() => import('../risk/Risk').then(m => ({ default: m.Risk })))
const Notifications = lazy(() => import('../notifications/Notifications').then(m => ({ default: m.Notifications })))
const Security = lazy(() => import('../identity/Security').then(m => ({ default: m.Security })))
const Reporting = lazy(() => import('../reporting/Reporting').then(m => ({ default: m.Reporting })))
const Privacy = lazy(() => import('../privacy/Privacy').then(m => ({ default: m.Privacy })))
const icons = { security: <SafetyCertificateOutlined />, reporting: <DashboardOutlined />, privacy: <FileProtectOutlined />, campaigns: <DashboardOutlined />, adjustments: <FileProtectOutlined />, support: <TeamOutlined />, risk: <SafetyCertificateOutlined />, notifications: <BookOutlined />, commissions: <SafetyCertificateOutlined />, withdrawalRules: <FileProtectOutlined />, settlements: <SafetyCertificateOutlined />, taskRules: <FileProtectOutlined />, tasks: <SafetyCertificateOutlined />, catalog: <BookOutlined />, attribution: <QrcodeOutlined />, welcome: <DashboardOutlined />, rules: <FileProtectOutlined />, users: <TeamOutlined />, promotions: <QrcodeOutlined />, audit: <SafetyCertificateOutlined />, guide: <BookOutlined /> }
interface Pending { action: string; body: Record<string, unknown>; key: string; owner: string }
function currentSection(): Section { return sections.find(section => `#/${section.key}` === window.location.hash)?.key || 'welcome' }
async function readStaffSession() {
  const user = await identity.me()
  if (user.status !== 'active' || !user.roles.some(role => role !== 'user')) {
    setToken('')
    throw new Error('此账号未获有效管理后台岗位授权')
  }
  const rules = user.roles.includes('rule_manager') ? await governance.rules() : []
  return { user, rules }
}
export function Console() {
 const [reauthOpen,setReauthOpen]=useState(false),[reauthPassword,setReauthPassword]=useState('')
  const [restoring, setRestoring] = useState(hasToken)
  const [restoreError, setRestoreError] = useState('')
  const [restoreAttempt, setRestoreAttempt] = useState(0)
  const [me, setMe] = useState<User | null>(null)
  const [rules, setRules] = useState<Rule[]>([])
  const [section, setSection] = useState<Section>(currentSection)
  const [notice, setNotice] = useState<{ type: 'success' | 'error'; text: string }>()
  const [busy, setBusy] = useState(false)
  const locked = useRef(false)
  const [pending, setPending] = useState<Pending | null>(null)
  useEffect(() => { const onHash = () => setSection(currentSection()); window.addEventListener('hashchange', onHash); return () => window.removeEventListener('hashchange', onHash) }, [])
  useEffect(() => {
    if (!hasToken()) { setRestoring(false); return }
    let active = true
    void readStaffSession().then(session => {
      if (!active) return
      setMe(session.user); setRules(session.rules); setRestoreError('')
    }).catch((error: unknown) => {
      if (!active) return
      const text = error instanceof Error ? error.message : '恢复登录状态失败，请重试'
      if (!hasToken()) setNotice({ type: 'error', text })
      else setRestoreError(text)
    }).finally(() => { if (active) setRestoring(false) })
    return () => { active = false }
  }, [restoreAttempt])
  function navigate(next: Section) { setSection(next); window.location.hash = `/${next}`; setNotice(undefined) }
  function clearSession(keepPending = false) { setToken(''); setMe(null); setRules([]); setNotice(undefined); if (!keepPending) setPending(null) }
  async function run(work: () => Promise<void>): Promise<boolean> {
    if (locked.current) return false
    locked.current = true; setBusy(true); setNotice(undefined)
    try { await work(); setNotice({ type: 'success', text: '操作完成' }); return true }
    catch (error) {
      if (error instanceof BusinessError && error.detail.code === 'SESSION_EXPIRED') clearSession(true)
      if (error instanceof BusinessError && error.detail.code === 'REAUTH_REQUIRED') setReauthOpen(true)
      setNotice({ type: 'error', text: error instanceof Error ? error.message : '操作失败，请查询原结果后重试' }); return false
    } finally { locked.current = false; setBusy(false) }
  }
  async function refresh() {
    const user = await identity.me()
    const nextRules = user.roles.includes('rule_manager') ? await governance.rules() : []
    setMe(user); setRules(nextRules)
  }
  async function save(action: string, body: Record<string, unknown>) {
    const saved = await run(async () => {
      if (pending) throw new Error('请先查询上次提交结果，或使用原请求重试')
      const next = { action, body, key: crypto.randomUUID(), owner: me?.id || '' }; setPending(next)
      try { await executeCommand(next.action, next.body, next.key); setPending(null) }
      catch (error) { if (error instanceof BusinessError && error.detail.code !== 'SERVICE_UNAVAILABLE') setPending(null); throw error }
    })
    // A failed refresh cannot turn a confirmed write into an unconfirmed submission.
    if (saved) await run(refresh)
    return saved
  }
  if (restoring) return <div className="session-loading" role="status"><Spin size="large" /><p>正在恢复登录状态…</p></div>
  if (restoreError) return <Result status="warning" title="暂时无法恢复登录状态" subTitle={restoreError} extra={<Space><Button type="primary" onClick={() => { setRestoring(true); setRestoreError(''); setRestoreAttempt(attempt => attempt + 1) }}>重试恢复</Button><Button onClick={() => { clearSession(); setRestoreError('') }}>返回登录</Button></Space>} />
  if (!me) return <AdminLogin initialError={notice?.type === 'error' ? notice.text : undefined} onAuthenticated={async session => {
    setToken(session.token)
    const { user, rules: nextRules } = await readStaffSession()
    if (pending && pending.owner !== session.user_id) setPending(null)
    setRules(nextRules); setMe(user); setNotice(undefined)
  }} />
  const available = sections.filter(item => (!item.role || me.roles.includes(item.role)) && (!item.roles || item.roles.some(role => me.roles.includes(role))))
  const active = available.find(item => item.key === section) || available[0]
  return <div className="admin-shell"><ProLayout title="Crowd" logo={<span className="brand-mark">C</span>} layout="mix" navTheme="light" fixSiderbar fixedHeader contentWidth="Fluid" location={{ pathname: `/${active.key}` }} route={{ path: '/', routes: available.map(item => ({ path: `/${item.key}`, name: item.title, icon: icons[item.key] })) }} menuItemRender={(item, dom) => <a href={`#${item.path}`} onClick={event => { event.preventDefault(); navigate(item.path?.slice(1) as Section) }}>{dom}</a>} avatarProps={{ icon: <UserOutlined />, title: me.nickname, render: (_, dom) => <Dropdown menu={{ items: [{ key: 'guide', label: '使用指南', icon: <BookOutlined /> }, { type: 'divider' }, { key: 'logout', label: '退出登录', icon: <LogoutOutlined />, disabled: busy }], onClick: ({ key }) => { if (key === 'guide') navigate('guide'); else if (key === 'logout') void run(async () => { try { await executeCommand('logout', {}, crypto.randomUUID()) } finally { clearSession() } }) } }}><span className="account-menu">{dom}</span></Dropdown> }} actionsRender={() => [<Tooltip title="刷新当前账号与规则" key="refresh"><Button type="text" aria-label="刷新当前账号与规则" icon={<ReloadOutlined spin={busy} />} disabled={busy} onClick={() => void run(refresh)} /></Tooltip>, <Tooltip title="使用指南" key="guide"><Button type="text" aria-label="使用指南" icon={<BookOutlined />} onClick={() => navigate('guide')} /></Tooltip>]} footerRender={() => <footer className="workspace-footer">Crowd · 产品推广与协作平台 <span>业务数字记录 · 人工线下结算</span></footer>}>
    <PageContainer title={active.title} content={active.description} breadcrumb={{ items: [{ title: '运营管理' }, { title: active.title }] }} extra={<Tag color="blue">{me.status === 'active' ? '正常工作中' : '账号受限'}</Tag>}>
      <Space orientation="vertical" size={20} className="full-width">
        <Button disabled={busy} onClick={() => setReauthOpen(true)}>高风险操作：再次验证当前会话</Button>
        <Modal title="再次验证当前后台会话" open={reauthOpen} onCancel={() => { setReauthOpen(false); setReauthPassword('') }} confirmLoading={busy} onOk={() => void run(async () => { try { await identity.reauthenticate(reauthPassword); setReauthOpen(false) } finally { setReauthPassword('') } })}><Input.Password aria-label="后台密码" autoComplete="current-password" value={reauthPassword} onChange={e => setReauthPassword(e.target.value)} /></Modal>
        {notice && <Alert type={notice.type} showIcon title={notice.text} closable onClose={() => setNotice(undefined)} />}
        {pending && pending.owner === me.id && <Alert type="warning" showIcon title="上次提交结果尚未确认" description="请先查询原结果；重试将使用原动作、内容和请求键。" action={<Space wrap><Button loading={busy} onClick={() => void run(async () => { await platform.commandResult(pending.key); setPending(null); await refresh() })}>查询原结果</Button><Button disabled={busy} onClick={() => void run(async () => { await executeCommand(pending.action, pending.body, pending.key); setPending(null); await refresh() })}>原请求重试</Button></Space>} />}
        <Spin spinning={busy} delay={300}><Suspense fallback={<div className="page-loading"><Spin /></div>}>
          {active.key === 'welcome' && <Welcome me={me} rules={rules} navigate={navigate} />}
          {active.key === 'rules' && <RuleCenter rules={rules} busy={busy} pending={!!pending} save={save} />}
          {active.key === 'users' && <UserManagement actorId={me.id} busy={busy} pending={!!pending} run={run} save={save} />}
          {active.key === 'promotions' && <Promotions busy={busy} pending={!!pending} run={run} save={save} />}
          {active.key === 'audit' && <AuditLog busy={busy} run={run} />}
          {active.key === 'catalog' && <Catalog me={me} busy={busy} pending={!!pending} run={run} save={save} />}
          {active.key === 'attribution' && <Attribution me={me} busy={busy} pending={!!pending} run={run} save={save} />}
          {active.key === 'taskRules' && <TaskRules me={me} busy={busy} pending={!!pending} run={run} save={save} />}
          {active.key === 'tasks' && <Tasks me={me} busy={busy} pending={!!pending} run={run} save={save} />}
          {active.key === 'commissions' && <Commissions me={me} busy={busy} pending={!!pending} run={run} save={save} />}
          {active.key === 'withdrawalRules' && <WithdrawalRules me={me} busy={busy} pending={!!pending} run={run} save={save} />}
          {active.key === 'settlements' && <Settlements me={me} busy={busy} pending={!!pending} run={run} save={save} />}
          {active.key === 'campaigns' && <Campaigns locked={busy || !!pending} run={run} save={save} />}
          {active.key === 'adjustments' && <Adjustments locked={busy || !!pending} run={run} save={save} />}
          {active.key === 'support' && <Support locked={busy || !!pending} run={run} save={save} />}
          {active.key === 'risk' && <Risk locked={busy || !!pending} run={run} save={save} />}
          {active.key === 'notifications' && <Notifications locked={busy || !!pending} run={run} save={save} />}
          {active.key === 'security' && <Security locked={busy || !!pending} run={run} save={save} />}
          {active.key === 'reporting' && <Reporting locked={busy || !!pending} run={run} save={save} />}
          {active.key === 'privacy' && <Privacy locked={busy || !!pending} run={run} save={save} />}
          {active.key === 'guide' && <Guide />}
        </Suspense></Spin>
      </Space>
    </PageContainer>
  </ProLayout></div>
}
