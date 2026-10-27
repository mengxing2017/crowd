export interface ApiError { code: string; message: string; request_id: string }
export interface Envelope<T> { data: T; request_id: string }
export interface Receipt { id: string; method?: 'wechat' | 'alipay'; account?: string; file_id?: string; status: string; version?: number; disables?: string }
export interface User { id: string; nickname: string; avatar: string | null; version: number; status: string; roles: string[]; scope: string[]; receipts: Receipt[]; restrictions: { scope: string; reason: string; appeal: string }[] }
export interface Rule { id: string; kind: string; version: number; status: string; content: Record<string, unknown> }
export interface Promotion { id: string; code: { owner: string; version: number; active: boolean; product: string | null; task: string | null } }
export interface Login { token: string; user_id: string; expires_in: number }
export interface ProductImage { file_id: string; alt: string }
export interface ProductTask { template_id?: string; price_id?: string; template_preview?: TaskTemplate; price_preview?: PriceRule; id: string; name: string; version: number; completion: string; evidence: string; rule_version: string; commission_cents: number; currency: 'CNY'; rule_starts_at: number; rule_ends_at: number; allow_self: boolean; submit_seconds: number; capacity?: number | null; digital_limit_cents?: number | null }
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
    if (request.path.startsWith('/api/v1/community/read/my_tasks') || request.path.startsWith('/api/v1/community/read/task_queue')) {
      if (!record(value) || !Array.isArray(value.items) || !(value.next_cursor === null || typeof value.next_cursor === 'string')) throw new Error('任务列表响应不符合契约')
      for (const item of value.items) if (!record(item) || typeof item.id !== 'string' || !Number.isSafeInteger(item.version) || typeof item.status !== 'string' || !record(item.template) || !Number.isSafeInteger(item.reserved_cents)) throw new Error('任务实例响应不符合契约')
    }
    if (request.path.startsWith('/api/v1/community/read/task_rules')) {
      if (!record(value) || !Array.isArray(value.prices) || !Array.isArray(value.templates) || !Number.isSafeInteger(value.server_time)) throw new Error('任务规则响应不符合契约')
      for (const rule of [...value.prices, ...value.templates]) if (!record(rule) || typeof rule.id !== 'string' || !Number.isSafeInteger(rule.version) || typeof rule.status !== 'string' || !record(rule.content)) throw new Error('任务规则版本不符合契约')
      for (const rule of value.prices) { const content = (rule as { content: Record<string, unknown> }).content; if (content.currency !== 'CNY' || (content.kind === 'fixed' && !Number.isSafeInteger(content.amount_cents)) || (content.kind === 'proportional' && (!Number.isSafeInteger(content.numerator) || !Number.isSafeInteger(content.denominator)))) throw new Error('佣金定价响应不符合契约') }
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

export interface PriceRule { kind: 'fixed' | 'proportional'; task_id: string; settlement_scope: string; currency: 'CNY'; starts_at: number; ends_at: number; amount_cents?: number; numerator?: number; denominator?: number; rounding?: 'floor' | 'ceil' | 'half_up'; base_source?: string; cap_cents?: number; period_cap_cents?: number; period_seconds?: number; period_anchor?: number; reservation_policy?: 'reserve_max_release_terminal'; group?: string }
export interface EvidenceField { id: string; label: string; purpose: string; kind: 'text' | 'image' | 'order'; required: boolean; max_length?: number; source?: string; exclusive_scope?: string }
export interface TaskTemplate { name: string; task_type: string; completion: string; failure_conditions: string; review_goal: string; source: 'manual' | 'external' | 'both'; external_source?: string; steps: { id: string; title: string; required: boolean }[]; fields: EvidenceField[]; regions: string[]; qualifications: string[]; execution_seconds: number; submit_seconds: number; claim_until: number; total_quota: number; per_user_quota: number; participation: 'once' | 'period' | 'repeat'; period_seconds?: number; period_anchor?: number; allow_abandon: boolean; release_on_terminal: boolean; allow_withdraw: boolean; pause_policy: 'extend' | 'keep_deadline'; double_review: boolean; max_supplements: number; supplement_seconds: number; review_sla_seconds: number; max_evidence_files: number; max_file_bytes: number }
export interface VersionedTaskRule<T> { id: string; version: number; status: 'pending' | 'approved' | 'rejected' | 'disabled'; editor: string; content: T; reason?: string }
export interface TaskRules { prices: VersionedTaskRule<PriceRule>[]; templates: VersionedTaskRule<TaskTemplate>[]; server_time: number }
export interface Evidence { fields: Record<string, string | string[]>; occurred_at?: number }
export interface TaskSubmission { id: string; version: number; evidence: Evidence; submitted_at: number; declaration_confirmed: boolean }
export interface TaskInstance extends Claim { corrections?: { id: string; kind: string; status: string; actor: string; reason: string }[]; version: number; template_id: string; template: TaskTemplate; price: PriceRule; started_at?: number; execute_until: number; steps: string[]; draft?: Evidence; preview?: { id: string; evidence: Evidence; at: number }; receipt?: { id: string; version: number; submitted_at: number; evidence?: Evidence | null }; submissions: TaskSubmission[] | null; reviewer?: string; first_reviewer?: string; supplement_fields?: string[]; supplement_until?: number; decisions: { actor: string; reason: string; decision: string }[]; reserved_cents: number; reservation_released: boolean; overdue?: boolean; waiting_seconds?: number; events: { action: string; actor: string; at: number; reason?: string }[] }
export interface TaskPage { items: TaskInstance[]; next_cursor: string | null; server_time: number }
export const taskStatusNames: Record<string, string> = { claimed: '已领取', in_progress: '执行中', paused: '已暂停', submitted: '待审核', reviewing: '审核中', supplement: '待补交', awaiting_verification: '待独立复核', approved: '审核通过（数字确认待入账）', rejected: '未通过', abandoned: '已放弃', expired: '已失效', disputed: '争议核查中' }
