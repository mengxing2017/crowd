import type { Request } from '../common/types.js'
import { record } from '../common/validation.js'
function receipt(value: unknown): boolean {
  return record(value) && (value.is_default === undefined || typeof value.is_default === 'boolean') && (value.declaration_version === undefined || typeof value.declaration_version === 'string') && (value.confirmed_at === undefined || typeof value.confirmed_at === 'string') && typeof value.id === 'string' && typeof value.status === 'string' && (value.method === undefined || value.method === 'wechat' || value.method === 'alipay') && (value.version === undefined || (Number.isSafeInteger(value.version) && Number(value.version) >= 1))
}
export function validateReceipts(request: Request, value: unknown): void {
  const path = request.path.split('?')[0]
  if (path === '/api/v1/community/read/receipts' && (!Array.isArray(value) || !value.every(receipt))) throw new Error('收款资料响应不符合契约')
  if (path === '/api/v1/community/commands/receipt_reveal' && !receipt(value)) throw new Error('收款资料响应不符合契约')
}
