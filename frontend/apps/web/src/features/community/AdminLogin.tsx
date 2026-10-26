import { useState, type FormEvent } from 'react'
import { community, setToken, type Login } from './api'

export function AdminLogin({ onAuthenticated }: { onAuthenticated: (session: Login) => Promise<void> }) {
  const [mode, setMode] = useState<'password' | 'ticket'>('password')
  const [username, setUsername] = useState('admin')
  const [password, setPassword] = useState('')
  const [ticket, setTicket] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (busy) return
    setBusy(true); setError('')
    try {
      const session = mode === 'password' ? await community.adminLogin({ username, password }) : await community.redeem(ticket)
      await onAuthenticated(session)
      setPassword(''); setTicket('')
    } catch (e) {
      setToken('')
      setError(e instanceof Error ? e.message : '登录失败，请稍后重试')
    } finally { setBusy(false) }
  }
  return <main className="admin-login">
    <div className="login-intro"><span className="login-brand">CROWD</span><h1>让推广协作<br />有序发生。</h1><p>业务规则、用户管理与推广记录，<br />在同一个工作台中处理。</p><small>佣金仅为业务数字，实际付款由人员在线下完成。</small></div>
    <section className="login-panel"><h2>管理后台登录</h2><p>使用管理员账号登录，进入岗位工作台。</p>
      <form onSubmit={event => void submit(event)} aria-busy={busy}>
        <fieldset disabled={busy}>
          {mode === 'password' ? <><label htmlFor="admin-username">管理员账号<input id="admin-username" name="username" autoComplete="username" required maxLength={64} value={username} onChange={e => setUsername(e.target.value)} /></label><label htmlFor="admin-password">密码<input id="admin-password" name="password" type="password" autoComplete="current-password" required maxLength={128} value={password} onChange={e => setPassword(e.target.value)} /></label></> : <><p>在已登录的小程序个人中心生成一次性票据，有效期 60 秒。</p><label htmlFor="admin-ticket">一次性登录票据<input id="admin-ticket" name="ticket" autoComplete="off" required value={ticket} onChange={e => setTicket(e.target.value)} /></label></>}
          {error && <p className="login-error" role="alert">{error}</p>}
          <button className="login-submit" type="submit">{busy ? '正在登录…' : '登录管理后台'}</button>
          <button className="login-switch" type="button" onClick={() => { setMode(mode === 'password' ? 'ticket' : 'password'); setPassword(''); setTicket(''); setError('') }}>{mode === 'password' ? '使用小程序一次性票据登录' : '使用管理员账号密码登录'}</button>
        </fieldset>
      </form><small>仅限已获授权的工作人员使用。登录后仍按岗位和数据范围授权。</small>
    </section>
  </main>
}
