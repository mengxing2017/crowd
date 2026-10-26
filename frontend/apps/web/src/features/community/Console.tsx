import { useState } from 'react'
import { community, setToken } from './api/client'
import { BusinessError, type Rule, type User } from './api'
const kinds: Record<string, string> = { business_chain: '业务闭环定义', operator: '平台责任主体', roles: '角色职责', completion: '任务完成口径', review: '平台审核口径', product_tasks: '产品与任务关系', attribution: '推广关系边界', digital_boundary: '数字记录边界', amount_unit: '整数分与展示单位', release_scope: '首期范围与变更影响', agreements: '服务协议与隐私说明' }
export function Console() {
  const [ticket, setTicket] = useState('')
  const [me, setMe] = useState<User | null>(null)
  const [rules, setRules] = useState<Rule[]>([])
  const [kind, setKind] = useState('operator')
  const [content, setContent] = useState('{}')
  const [target, setTarget] = useState('')
  const [action, setAction] = useState('users')
  const [input, setInput] = useState('{}')
  const [result, setResult] = useState<unknown>(null)
  const [message, setMessage] = useState('')
  const [busy, setBusy] = useState(false)
  const [pending, setPending] = useState<{ action: string; body: Record<string, unknown>; key: string; owner: string } | null>(null)
  async function run(work: () => Promise<void>) { setBusy(true); setMessage('处理中…'); try { await work(); setMessage('操作完成') } catch (e) { if (e instanceof BusinessError && e.detail.code === 'SESSION_EXPIRED') { setMe(null); setToken(''); setRules([]); setResult(null); setInput('{}'); setContent('{}'); setTarget('') } setMessage(e instanceof Error ? e.message : '操作失败，请查询原结果') } finally { setBusy(false) } }
  async function refresh() { const user = await community.read<User>('me'); setMe(user); if (user.roles.includes('rule_manager')) setRules(await community.read<Rule[]>('rules')) }
  async function submit(name: string, body: Record<string, unknown>) {
    const next = { action: name, body, key: crypto.randomUUID(), owner: me?.id || '' }; setPending(next)
    try { const value = await community.command(next.action, next.body, next.key); setResult(value); setPending(null); await refresh() } catch (e) { if (e instanceof BusinessError && !['SERVICE_UNAVAILABLE'].includes(e.detail.code)) setPending(null); throw e }
  }
  return <main><h1>Crowd 运营管理</h1><p>佣金仅为业务数字。付款由人员在线下完成，本系统不提供支付或资金托管。</p>
    <p role="status">{message}</p>
    {!me ? <section><h2>安全登录</h2><p>在已登录的小程序个人中心生成一次性后台登录票据，有效期 60 秒。后台功能仍由服务端逐项授权。</p><label>一次性票据<input value={ticket} onChange={e => setTicket(e.target.value)} autoComplete="off" /></label><button disabled={busy || !ticket} onClick={() => void run(async () => { const session = await community.redeem(ticket); if (pending && pending.owner !== session.user_id) setPending(null); setToken(session.token); setTicket(''); setMe(await community.read<User>('me')); try { setRules(await community.read<Rule[]>('rules')) } catch { /* Other roles may not manage rules. */ } })}>登录</button></section> : <>
      <p>{me.nickname} · {me.status} · {me.roles.join('、')}</p><button disabled={busy} onClick={() => void run(async () => { try { await community.command('logout', {}, crypto.randomUUID()) } finally { setToken(''); setMe(null); setRules([]); setResult(null); setPending(null); setContent('{}'); setInput('{}'); setTarget('') } })}>退出登录并清理缓存</button>
      {me.roles.includes('rule_manager') && <section><h2>业务规则中心</h2><label>规则类型<select value={kind} onChange={e => setKind(e.target.value)}>{Object.entries(kinds).map(([key, label]) => <option key={key} value={key}>{label}</option>)}</select></label><p>保存为草稿后单独发布；未确认限制保留在待确认项中。协议内容字段为 service、privacy；数字边界必须 manual_only: true。产品任务目录格式：products 数组，产品含 id、status、tasks；任务含 id、status、completion、evidence。状态为 active 或 inactive。</p><label>版本内容（JSON 对象）<textarea rows={8} value={content} onChange={e => setContent(e.target.value)} /></label><button disabled={busy || !!pending} onClick={() => void run(async () => { await submit('rule_save', { kind, content: JSON.parse(content) as Record<string, unknown>, version: rules.filter(r => r.kind === kind).length }) })}>保存新草稿</button>
      {rules.length === 0 && <p>暂无规则版本。</p>}{rules.map(rule => <article key={rule.id}><h3>{kinds[rule.kind]} · V{rule.version} · {rule.status}</h3><pre>{JSON.stringify(rule.content, null, 2)}</pre>{rule.status !== 'disabled' && <button disabled={busy || !!pending} onClick={() => void run(async () => { await submit(rule.status === 'draft' ? 'rule_publish' : 'rule_disable', { id: rule.id, version: rule.version }) })}>{rule.status === 'draft' ? '发布此版本' : '停用此版本'}</button>}</article>)}</section>}
      <section><h2>岗位操作与审计</h2><p>服务端检查动作权限及对象范围；不具备权限的请求会被拒绝。完整收款资料必须由申请服务提供授权上下文，目前不向后台开放。</p><label>对象用户或审计对象<input value={target} onChange={e => setTarget(e.target.value)} /></label><label>操作<select value={action} onChange={e => setAction(e.target.value)}><option value="users">查询用户权限与状态</option><option value="audit">查询审计</option><option value="managed_codes">查询用户推广码</option><option value="access">配置角色与数据范围</option><option value="restriction">限制账号</option><option value="code_rotate">撤销并换发推广码</option></select></label><label>操作参数（JSON 对象，修改须包含 version、reason；换码须包含 id、approval）<textarea rows={6} value={input} onChange={e => setInput(e.target.value)} /></label><button disabled={busy || !!pending} onClick={() => void run(async () => { if (['users', 'audit', 'managed_codes'].includes(action)) setResult(await community.read(action, target)); else await submit(action, { ...(JSON.parse(input) as Record<string, unknown>), user_id: target }) })}>执行</button></section>
      {pending && <section><p>上次提交结果尚未确认，请先查询原结果。重试会使用原请求键与原内容。</p><button disabled={busy} onClick={() => void run(async () => { setResult(await community.read('command_result', pending.key)); setPending(null); await refresh() })}>查询原结果</button><button disabled={busy} onClick={() => void run(async () => { setResult(await community.command(pending.action, pending.body, pending.key)); setPending(null); await refresh() })}>重试原提交</button></section>}
      {result !== null && <section><h2>操作结果</h2><pre>{JSON.stringify(result, null, 2)}</pre></section>}
    </>}
  </main>
}
