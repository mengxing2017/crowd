import { useEffect, useState } from 'react'
import { CopyOutlined, QrcodeOutlined } from '@ant-design/icons'
import { Alert, Button, Card, Result, Spin, Typography } from 'antd'
import { community } from './api'
export function Landing({ code }: { code: string }) {
  const [notice, setNotice] = useState('')
  const [state, setState] = useState<'loading' | 'valid' | 'error'>('loading')
  const [copy, setCopy] = useState('')
  useEffect(() => { let active = true; setState('loading'); setCopy(''); void community.resolve(code).then(result => { if (active) { setState('valid'); setNotice(result.notice) } }).catch((error: unknown) => { if (active) { setState('error'); setNotice(error instanceof Error ? error.message : '入口暂不可用') } }); return () => { active = false } }, [code])
  return <main className="promotion-landing"><Card><Spin spinning={state === 'loading'}><Result status={state === 'error' ? 'warning' : 'info'} icon={state === 'error' ? undefined : <QrcodeOutlined />} title={state === 'loading' ? '正在解析推广入口' : state === 'valid' ? 'Crowd 推广邀请' : '推广入口暂不可用'} subTitle={notice} extra={state === 'valid' ? <><Typography.Paragraph>打开 Crowd 微信小程序，在注册登录页填写推广来源码。</Typography.Paragraph><Typography.Paragraph className="promotion-code">{code}</Typography.Paragraph><Button type="primary" icon={<CopyOutlined />} onClick={() => { void navigator.clipboard.writeText(code).then(() => setCopy('已复制推广码')).catch(() => setCopy('复制失败，请手动复制页面中的推广码')) }}>复制推广码</Button><p role="status">{copy}</p></> : undefined} /></Spin><Alert type="info" showIcon title="来源码仅记录注册入口" description="不代表任务完成或获得佣金；佣金仅为业务数字，实际付款在线下完成。" /></Card></main>
}
