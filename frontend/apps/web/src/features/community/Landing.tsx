import { useEffect, useState } from 'react'
import { community } from './api/client'
export function Landing({ code }: { code: string }) {
  const [message, setMessage] = useState('正在解析推广入口…')
  const [valid, setValid] = useState(false)
  useEffect(() => { let active = true; void community.resolve(code).then(result => { if (active) { setValid(true); setMessage(result.notice) } }).catch((e: unknown) => { if (active) setMessage(e instanceof Error ? e.message : '入口暂不可用') }); return () => { active = false } }, [code])
  return <main><h1>Crowd 推广入口</h1><p role="status">{message}</p>{valid && <section><p>推广码：{code}</p><p>打开 Crowd 微信小程序，在注册登录页填写此推广来源码。来源码只记录注册入口，不代表任务已完成或获得佣金。</p><button onClick={() => { void navigator.clipboard.writeText(code).then(() => setMessage('已复制推广码')).catch(() => setMessage('复制失败，请手动复制页面中的推广码')) }}>复制推广码</button></section>}</main>
}
