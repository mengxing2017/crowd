import type { ApiError, Request, Transport } from './types.js'
import { BusinessError } from './errors.js'

export function unwrapEnvelope<T>(value: unknown): T {
  const raw = value as { data?: T; error?: ApiError; request_id?: string } | null
  if (raw?.error && typeof raw.error.code === 'string' && typeof raw.error.message === 'string' && typeof raw.error.request_id === 'string') throw new BusinessError(raw.error)
  if (!raw || typeof raw !== 'object' || !('data' in raw) || typeof raw.request_id !== 'string') throw new Error('服务响应不符合契约，请查询原结果后重试')
  return raw.data as T
}
export function createRequester(transport: Transport, validateResponse: (request: Request, value: unknown) => void = () => {}) {
  return async function call<T>(request: Request): Promise<T> {
    const value = unwrapEnvelope<T>(await transport(request))
    validateResponse(request, value)
    return value
  }
}
