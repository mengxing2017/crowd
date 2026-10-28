import type { Transport } from '../common/types.js'
import { createRequester } from '../common/request.js'
import { createResource } from '../common/resource.js'
import type { Promotion, ResolvedPromotion } from './types.js'
import { validatePromotion } from './validation.js'

export function createPromotion(transport: Transport) {
  const resource = createResource(transport, validatePromotion)
  const call = createRequester(transport, validatePromotion)
  return {
    resolve: (code: string) => call<ResolvedPromotion>({ method: 'GET', path: `/api/v1/community/resolve/${encodeURIComponent(code)}` }),
    codes: () => resource.read<Promotion[]>('codes'),
    managedCodes: (owner: string) => resource.read<Promotion[]>('managed_codes', owner),
    codeCreate: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('code_create', body, key),
    codeRotate: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('code_rotate', body, key),
  }
}
