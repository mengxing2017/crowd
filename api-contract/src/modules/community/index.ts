export interface ApiError { code: string; message: string; request_id: string }
export interface Envelope<T> { data: T; request_id: string }
export interface Receipt { id: string; method?: 'wechat' | 'alipay'; account?: string; file_id?: string; status: string; version?: number; disables?: string }
export interface User { id: string; nickname: string; avatar: string | null; version: number; status: string; roles: string[]; scope: string[]; receipts: Receipt[]; restrictions: { scope: string; reason: string; appeal: string }[] }
export interface Rule { id: string; kind: string; version: number; status: string; content: Record<string, unknown> }
export interface Promotion { id: string; code: { owner: string; version: number; active: boolean; product: string | null; task: string | null } }
export interface Login { token: string; user_id: string; expires_in: number }
export interface ProductImage { file_id: string; alt: string }
export interface ProductTask { id: string; name: string; version: number; completion: string; evidence: string; rule_version: string; commission_cents: number; currency: 'CNY'; rule_starts_at: number; rule_ends_at: number; allow_self: boolean; submit_seconds: number; capacity?: number | null; digital_limit_cents?: number | null }
export interface ProductContent { name: string; category: string; summary: string; details: string; limitations: string; estimated_minutes: number; regions: string[]; qualifications: string[]; images: ProductImage[]; tasks: ProductTask[] }
export interface Category { id: string; version: number; name: string; enabled: boolean; sort: number; reason: string }
export interface ProductRelease { id: string; content: ProductContent; starts_at: number; ends_at: number; status: 'pending' | 'approved' | 'rejected' | 'withdrawn'; submitted_by: string; editor: string; review_reason?: string }
export interface Product { id: string; version: number; draft: Partial<ProductContent>; releases: ProductRelease[]; archived: boolean; offline: boolean }
export interface CatalogAdmin { categories: Category[]; products: Product[]; revision: number; server_time: number }
export interface PublicProduct { id: string; release_id: string; content: ProductContent; starts_at: number; ends_at: number; available_tasks: string[] }
export type PublicCategory = Pick<Category, 'id' | 'version' | 'name' | 'sort' | 'enabled'>
export interface PublicCatalog { categories: PublicCategory[]; products: PublicProduct[]; revision: number }
export interface Claim { id: string; owner: string; product_id: string; release_id: string; product: ProductContent; category: Category; task: ProductTask; policy: AttributionPolicy; touch: { promoter: string; code: string; time: number } | null; locked_at: number; submit_until: number; status: string }
export interface AttributionPolicy { id: string; version: number; window_seconds: number; strategy: 'first_touch' | 'last_touch' }
export interface AttributionDispute { id: string; version: number; owner: string; claim_id: string; reason: string; evidence: string; status: string; decision_reason?: string }
export interface AttributionAdmin { policies: AttributionPolicy[]; touch_count: number; disputes: AttributionDispute[] }
export interface CatalogNotification { recipient: string; product_id: string; kind: string; time: number; revision: number }
export interface CatalogQuery { keyword?: string; category?: string; min_cents?: number; max_cents?: number; sort?: 'name' | 'commission_asc' | 'commission_desc'; task_status?: 'available' | 'unavailable' }
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
    if (request.path === '/api/v1/community/admin/login' || request.path === '/api/v1/community/login' || request.path === '/api/v1/community/session') guardLogin(value)
    if (request.path.startsWith('/api/v1/catalog') && request.method === 'GET') {
      if (!record(value) || !Array.isArray(value.products) || !Array.isArray(value.categories) || !Number.isSafeInteger(value.revision)) throw new Error('产品目录响应不符合契约')
      for (const product of value.products) {
        if (!record(product) || typeof product.id !== 'string' || typeof product.release_id !== 'string' || !record(product.content) || typeof product.content.name !== 'string' || !Array.isArray(product.content.tasks) || !Array.isArray(product.available_tasks)) throw new Error('产品响应不符合契约')
        for (const task of product.content.tasks) if (!record(task) || !Number.isSafeInteger(task.commission_cents)) throw new Error('佣金数字响应不符合契约')
      }
    }
    return value as T
  }
  return {
    catalog: (query: CatalogQuery = {}) => {
      const params = Object.entries(query).filter(([, value]) => value !== undefined && value !== '').map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`).join('&')
      return call<PublicCatalog>({ method: 'GET', path: `/api/v1/catalog${params ? `?${params}` : ''}` })
    },
    touch: (body: { code: string; scene: string }, key: string) => call<{ touch_token: string; product: string | null; task: string | null; notice: string }>({ method: 'POST', path: '/api/v1/catalog/touches', body, key }),
    agreements: () => call<Rule>({ method: 'GET', path: '/api/v1/community/agreements' }),
    login: (body: { code: string; agreed: boolean; agreement_id?: string; source?: string; touch_token?: string }) => call<Login>({ method: 'POST', path: '/api/v1/community/login', body }),
    adminLogin: (body: { username: string; password: string }) => call<Login>({ method: 'POST', path: '/api/v1/community/admin/login', body }),
    redeem: (ticket: string) => call<Login>({ method: 'POST', path: '/api/v1/community/session', body: { ticket } }),
    read: <T>(section: string, target?: string) => call<T>({ method: 'GET', path: `/api/v1/community/read/${encodeURIComponent(section)}${target ? `?target=${encodeURIComponent(target)}` : ''}` }),
    command: <T = Record<string, unknown>>(action: string, body: Record<string, unknown>, key: string) => call<T>({ method: 'POST', path: `/api/v1/community/commands/${encodeURIComponent(action)}`, body, key }),
    resolve: (code: string) => call<{ code: string; notice: string }>({ method: 'GET', path: `/api/v1/community/resolve/${encodeURIComponent(code)}` }),
  }
}
