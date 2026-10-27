export { createCommunity, BusinessError } from '@crowd/api-contract/community'
export type { User, Rule, Promotion, Receipt, Login, Request, Product, ProductContent, ProductTask, ProductRelease, Category, CatalogAdmin, PublicProduct, PublicCatalog, Claim, AttributionAdmin, AttributionPolicy, AttributionDispute, CatalogNotification } from '@crowd/api-contract/community'
import { createCommunity } from '@crowd/api-contract/community'
const sessionKey = 'crowd.web.session'
function storedToken(): string {
  try {
    const value = window.sessionStorage.getItem(sessionKey) || ''
    if (/^[a-f0-9]{48}$/.test(value)) return value
    window.sessionStorage.removeItem(sessionKey)
  } catch { /* Storage restrictions still allow the current in-memory session. */ }
  return ''
}
let token = storedToken()
export function hasToken() { return token !== '' }
export function setToken(value: string) {
  token = value
  try {
    if (value) window.sessionStorage.setItem(sessionKey, value)
    else window.sessionStorage.removeItem(sessionKey)
  } catch { /* Keep login functional when browser storage is unavailable. */ }
}
export const community = createCommunity(async request => {
  const requestToken = token
  const result = await fetch(request.path, { method: request.method, headers: { 'Content-Type': 'application/json', ...(requestToken ? { Authorization: `Bearer ${requestToken}` } : {}), ...(request.key ? { 'Idempotency-Key': request.key } : {}) }, body: request.body === undefined ? undefined : JSON.stringify(request.body), cache: 'no-store' })
  if (result.status === 401 && token === requestToken) setToken('')
  return result.json() as Promise<unknown>
})

export async function uploadProductImage(file: File): Promise<string> {
  const response = await fetch('/api/v1/community/files/product_image', { method: 'POST', headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/octet-stream' }, body: file })
  const value = await response.json() as { data?: { id: string }; error?: { message: string; request_id: string } }
  if (!value.data) throw new Error(`${value.error?.message || '上传失败'}（${value.error?.request_id || ''}）`)
  return value.data.id
}
