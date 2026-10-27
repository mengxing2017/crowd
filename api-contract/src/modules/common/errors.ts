import type { ApiError } from './types.js'

export class BusinessError extends Error {
  constructor(public readonly detail: ApiError) { super(`${detail.message}（${detail.code}，追踪 ${detail.request_id}）`) }
}
