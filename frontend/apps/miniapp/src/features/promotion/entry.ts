// Avoid the browser URL API: the mini-program JS runtime is not a browser.
export function promotionSource(query: Record<string, unknown>, trustedEntry: string): string {
  if (typeof query.source === 'string' && !query.q) return query.source
  if (typeof query.q !== 'string') return ''
  const entry = trustedEntry.replace(/\/$/, '')
  if (!/^https:\/\/[^/?#]+(?:\/[^?#]*)?$/.test(entry)) throw new Error('尚未配置可信扫码入口')
  let link: string
  try { link = decodeURIComponent(query.q) } catch { throw new Error('二维码链接编码无效') }
  const start = `${entry}/?`
  if (!link.startsWith(start) || link.includes('#')) throw new Error('二维码不是受信任的推广入口')
  for (const pair of link.slice(start.length).split('&')) {
    const [key, ...value] = pair.split('=')
    if (key === 'source') {
      try { return decodeURIComponent(value.join('=').replace(/\+/g, ' ')) } catch { throw new Error('推广来源编码无效') }
    }
  }
  return ''
}
