export { createCommunity, BusinessError } from '@crowd/api-contract/community'
export type { User, Rule, Promotion, Receipt, Login, Request } from '@crowd/api-contract/community'
import { createCommunity } from '@crowd/api-contract/community'
let token = ''
export function setToken(value: string) { token = value }
export const community = createCommunity(async request => {
  const result = await fetch(request.path, { method: request.method, headers: { 'Content-Type': 'application/json', ...(token ? { Authorization: `Bearer ${token}` } : {}), ...(request.key ? { 'Idempotency-Key': request.key } : {}) }, body: request.body === undefined ? undefined : JSON.stringify(request.body), cache: 'no-store' })
  if (result.status === 401) setToken('')
  return result.json() as Promise<unknown>
})
