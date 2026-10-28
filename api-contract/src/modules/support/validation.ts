import type { Request } from '../common/types.js'
import { record } from '../common/validation.js'
export function validateSupport(request: Request, value: unknown): void {
  if (request.path.split('?')[0] === '/api/v1/community/read/cases' && (!Array.isArray(value) || !value.every(item => record(item) && ['id', 'kind', 'status'].every(field => typeof item[field] === 'string')))) throw new Error('客服申请响应不符合契约')
}
