export interface ApiError { code: string; message: string; request_id: string }
export interface Envelope<T> { data: T; request_id: string }
export interface Receipt { id: string; method?: 'wechat' | 'alipay'; account?: string; file_id?: string; status: string; version?: number; disables?: string }
export interface User { id: string; nickname: string; avatar: string | null; version: number; status: string; roles: string[]; scope: string[]; receipts: Receipt[]; restrictions: { scope: string; reason: string; appeal: string }[] }
export interface Rule { id: string; kind: string; version: number; status: string; content: Record<string, unknown> }
export interface Promotion { id: string; code: { owner: string; version: number; active: boolean; product: string | null; task: string | null } }
export interface Login { token: string; user_id: string; expires_in: number }
export interface Request { method: 'GET' | 'POST'; path: string; body?: unknown; key?: string }
export type Transport = (request: Request) => Promise<unknown>
export class BusinessError extends Error {
  constructor(public readonly detail: ApiError) { super(`${detail.message}（${detail.code}，追踪 ${detail.request_id}）`) }
}
function record(value: unknown): value is Record<string, unknown> { return typeof value === 'object' && value !== null && !Array.isArray(value) }
function guardUser(value: unknown): void {
  if (!record(value) || typeof value.id !== 'string' || typeof value.nickname !== 'string' || typeof value.version !== 'number' || !Number.isSafeInteger(value.version) || typeof value.status !== 'string' || !(value.avatar === null || typeof value.avatar === 'string') || !Array.isArray(value.roles) || !value.roles.every(v => typeof v === 'string') || !Array.isArray(value.scope) || !value.scope.every(v => typeof v === 'string') || !Array.isArray(value.receipts) || !Array.isArray(value.restrictions)) throw new Error('用户资料响应不符合契约')
}
function guardLogin(value: unknown): void {
  if (!record(value) || typeof value.token !== 'string' || value.token.length !== 48 || typeof value.user_id !== 'string' || typeof value.expires_in !== 'number' || value.expires_in <= 0) throw new Error('登录响应不符合契约')
}
export function createCommunity(transport: Transport) {
  async function call<T>(request: Request): Promise<T> {
    const raw = await transport(request) as { data?: T; error?: ApiError; request_id?: string } | null
    if (raw?.error && typeof raw.error.code === 'string' && typeof raw.error.message === 'string' && typeof raw.error.request_id === 'string') throw new BusinessError(raw.error)
    if (!raw || !('data' in raw) || typeof raw.request_id !== 'string') throw new Error('服务响应不符合契约，请查询原结果后重试')
    const value = raw.data
    if (request.path === '/api/v1/community/read/me') guardUser(value)
    if (request.path === '/api/v1/community/login' || request.path === '/api/v1/community/session') guardLogin(value)
    return value as T
  }
  return {
    agreements: () => call<Rule>({ method: 'GET', path: '/api/v1/community/agreements' }),
    login: (body: { code: string; agreed: boolean; agreement_id?: string; source?: string }) => call<Login>({ method: 'POST', path: '/api/v1/community/login', body }),
    redeem: (ticket: string) => call<Login>({ method: 'POST', path: '/api/v1/community/session', body: { ticket } }),
    read: <T>(section: string, target?: string) => call<T>({ method: 'GET', path: `/api/v1/community/read/${encodeURIComponent(section)}${target ? `?target=${encodeURIComponent(target)}` : ''}` }),
    command: <T = Record<string, unknown>>(action: string, body: Record<string, unknown>, key: string) => call<T>({ method: 'POST', path: `/api/v1/community/commands/${encodeURIComponent(action)}`, body, key }),
    resolve: (code: string) => call<{ code: string; notice: string }>({ method: 'GET', path: `/api/v1/community/resolve/${encodeURIComponent(code)}` }),
  }
}
