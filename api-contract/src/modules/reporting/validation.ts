import type { Request } from '../common/types.js'
import { record, safeIntegers } from '../common/validation.js'
export function validate(request: Request, value: unknown): void {
 if (!safeIntegers(value)) throw new Error('响应数字超出安全整数范围')
 if (request.method === 'POST' && (!record(value) || typeof value.id !== 'string')) throw new Error('操作响应缺少稳定编号')
 if (request.method === 'GET' && value !== null && !record(value) && !Array.isArray(value)) throw new Error('查询响应不符合契约')
}
