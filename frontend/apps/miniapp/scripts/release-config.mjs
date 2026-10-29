export function releaseConfig(env) {
  const appid = env.VITE_MP_WEIXIN_APPID || ''
  if (!/^wx[0-9a-f]{16}$/i.test(appid)) throw new Error('发布构建需要真实 VITE_MP_WEIXIN_APPID')
  function https(name, allowPath) {
    let url
    try { url = new URL(env[name]) } catch { throw new Error(`${name} 必须是有效 HTTPS 地址`) }
    if (url.protocol !== 'https:' || url.username || url.password || url.port || url.search || url.hash || /^(localhost|.*\.localhost|\d+(\.\d+){3}|\[.*\])$/i.test(url.hostname) || (!allowPath && url.pathname !== '/')) throw new Error(`${name} 必须使用无凭据、无端口的 HTTPS 域名${allowPath ? '入口' : '根地址'}`)
    return url.href.replace(/\/$/, '')
  }
  return { appid, origin: https('VITE_API_ORIGIN', false), promotion: https('VITE_PROMOTION_ORIGIN', true) }
}
