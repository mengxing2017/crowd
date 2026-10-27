import { useEffect, useRef, useState } from 'react'
import { CopyOutlined, QrcodeOutlined } from '@ant-design/icons'
import { Alert, Button, Card, Empty, Result, Space, Spin, Typography } from 'antd'
import { community, type PublicCatalog } from './api'
export function Landing({ code }: { code: string | null }) {
  const [notice, setNotice] = useState('')
  const [state, setState] = useState<'loading' | 'valid' | 'error' | 'catalog'>(code === null ? 'catalog' : 'loading')
  const [copy, setCopy] = useState('')
  const [catalog, setCatalog] = useState<PublicCatalog>()
  const [catalogError, setCatalogError] = useState('')
  const [showCatalog, setShowCatalog] = useState(code === null)
  const visit = useRef({ code, key: crypto.randomUUID() })
  useEffect(() => {
    if (code === null) return
    if (visit.current.code !== code) visit.current = { code, key: crypto.randomUUID() }
    let active = true; setState('loading'); setCopy('')
    // StrictMode retries share a key; a genuine page refresh receives a new visit key.
    void community.touch({ code, scene: 'web_share' }, visit.current.key).then(result => {
      if (active) { setState('valid'); setNotice(result.notice) }
    }).catch((error: unknown) => { if (active) { setState('error'); setNotice(error instanceof Error ? error.message : '入口暂不可用') } })
    return () => { active = false }
  }, [code])
  async function loadCatalog() { setCatalogError(''); try { setCatalog(await community.catalog()) } catch (error) { setCatalogError(error instanceof Error ? error.message : '目录暂不可用') } }
  useEffect(() => { if (showCatalog) void loadCatalog() }, [showCatalog])
  return <main className="promotion-landing"><Space orientation="vertical" size={20} className="full-width"><Card><Spin spinning={state === 'loading'}><Result status={state === 'error' ? 'warning' : 'info'} icon={state === 'error' ? undefined : <QrcodeOutlined />} title={state === 'loading' ? '正在解析推广入口' : state === 'valid' ? 'Crowd 推广邀请' : state === 'catalog' ? 'Crowd 产品目录' : '推广入口暂不可用'} subTitle={notice} extra={<Space orientation="vertical">{state === 'valid' && <><Typography.Paragraph>打开 Crowd 微信小程序，在注册登录页填写此来源码。登录时关联合法访问，领取时独立确认任务条件。</Typography.Paragraph><Typography.Paragraph className="promotion-code">{code}</Typography.Paragraph><Button type="primary" icon={<CopyOutlined />} onClick={() => { void navigator.clipboard.writeText(code || '').then(() => setCopy('已复制推广码')).catch(() => setCopy('复制失败，请手动复制页面中的推广码')) }}>复制推广码</Button><p role="status">{copy}</p></>}<Button onClick={() => { setShowCatalog(true); void loadCatalog() }}>返回产品列表</Button></Space>} /></Spin><Alert type="info" showIcon title="访问不等于领取或完成" description="扫码不占任务名额，也不产生佣金。佣金仅为业务数字，实际付款在线下完成。" /></Card>
    {showCatalog && <Card title="已发布产品" extra={<Button onClick={() => void loadCatalog()}>刷新产品列表</Button>}>{catalogError && <Alert type="error" title={catalogError} />}{!catalog && !catalogError ? <Spin /> : catalog?.products.length === 0 ? <Empty description="暂无已发布产品" /> : <Space orientation="vertical" className="full-width">{catalog?.products.map(p => <Card key={p.id} size="small" title={p.content.name}><Typography.Paragraph>{p.content.summary}</Typography.Paragraph><Typography.Paragraph>{p.content.details}</Typography.Paragraph><Alert type="warning" title={`关键限制：${p.content.limitations}`} />{p.content.tasks.map(t => <Typography.Paragraph key={t.id}>{t.name} · 佣金标记 {t.commission_cents} 分（CNY） · 条件：{t.completion} · 证据：{t.evidence}</Typography.Paragraph>)}<Typography.Paragraph type="secondary">请在微信小程序查看适用条件、选择任务并独立确认领取。</Typography.Paragraph></Card>)}</Space>}</Card>}
  </Space></main>
}
