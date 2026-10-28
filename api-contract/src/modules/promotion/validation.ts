import type { Request } from '../common/types.js'
import { record } from '../common/validation.js'
export function validatePromotion(request: Request, value: unknown): void {
  const path = request.path.split('?')[0]
  if (path === '/api/v1/community/read/codes' || path === '/api/v1/community/read/managed_codes') {
    if (!Array.isArray(value) || !value.every(item => record(item) && typeof item.id === 'string' && record(item.code) && typeof item.code.owner === 'string' && Number.isSafeInteger(item.code.version) && typeof item.code.active === 'boolean' && (item.code.product === null || typeof item.code.product === 'string') && (item.code.task === null || typeof item.code.task === 'string'))) throw new Error('推广码响应不符合契约')
  }
  if (path.startsWith('/api/v1/community/resolve/') && (!record(value) || typeof value.code !== 'string' || !Number.isSafeInteger(value.version) || typeof value.notice !== 'string' || !(value.product === null || typeof value.product === 'string') || !(value.task === null || typeof value.task === 'string'))) throw new Error('推广入口响应不符合契约')
}
