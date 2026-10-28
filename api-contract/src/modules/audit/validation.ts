import type { Request } from '../common/types.js'
import { record } from '../common/validation.js'
export function validateAudit(request: Request, value: unknown): void {
  if (request.path.split('?')[0] === '/api/v1/community/read/audit' && (!Array.isArray(value) || !value.every(item => record(item) && ['actor', 'action', 'object', 'request_id', 'time', 'result', 'rule_version'].every(field => typeof item[field] === 'string')))) throw new Error('审计记录响应不符合契约')
}
