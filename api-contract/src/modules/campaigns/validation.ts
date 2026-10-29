import { record } from '../common/validation.js'
import type { Request } from '../common/types.js'
function valid(value: unknown): boolean {
 if (typeof value === 'number') return Number.isSafeInteger(value)
 if (Array.isArray(value)) return value.every(valid)
 if (record(value)) return Object.values(value).every(valid)
 return true
}
export function validate(request: Request, value: unknown) {
 if (!valid(value)) throw new Error('服务响应包含不安全数字')
 if (request.method === 'GET' && !Array.isArray(value) && !record(value)) throw new Error('业务查询响应不符合契约')
 if (Array.isArray(value) && !value.every(item => record(item) && typeof item.id === 'string')) throw new Error('业务列表响应不符合契约')
 if (request.method === 'POST' && !record(value)) throw new Error('业务提交响应不符合契约')
}
