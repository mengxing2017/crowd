import type { Request } from '../common/types.js'
import { record } from '../common/validation.js'

function guardUser(value: unknown): void {
  if (!record(value) || typeof value.id !== 'string' || typeof value.nickname !== 'string' || typeof value.version !== 'number' || !Number.isSafeInteger(value.version) || typeof value.status !== 'string' || !(value.avatar === null || typeof value.avatar === 'string') || !Array.isArray(value.roles) || !value.roles.every(v => typeof v === 'string') || !Array.isArray(value.scope) || !value.scope.every(v => typeof v === 'string') || !Array.isArray(value.receipts) || !Array.isArray(value.restrictions)) throw new Error('用户资料响应不符合契约')
}
function guardLogin(value: unknown): void {
  if (!record(value) || typeof value.token !== 'string' || value.token.length !== 48 || typeof value.user_id !== 'string' || typeof value.expires_in !== 'number' || value.expires_in <= 0) throw new Error('登录响应不符合契约')
}

export function validateIdentity(request: Request, value: unknown): void {
  if (request.path === '/api/v1/community/read/me') guardUser(value)
  if (request.path === '/api/v1/community/admin/login' || request.path === '/api/v1/community/login' || request.path === '/api/v1/community/session') guardLogin(value)

}
