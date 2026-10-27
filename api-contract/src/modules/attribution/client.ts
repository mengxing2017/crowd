import type { Transport } from '../common/types.js'
import type { AttributionAdmin, Promotion } from './types.js'
import { createRequester } from '../common/request.js'
import { createResource } from '../common/resource.js'

export function createAttribution(transport: Transport) {
  const call = createRequester(transport)
  const resource = createResource(transport)
  return {
    touch: (body: { code: string; scene: string }, key: string) => call<{ touch_token: string; product: string | null; task: string | null; notice: string }>({ method: 'POST', path: '/api/v1/catalog/touches', body, key }),
    resolve: (code: string) => call<{ code: string; notice: string }>({ method: 'GET', path: `/api/v1/community/resolve/${encodeURIComponent(code)}` }),
    codeCreate: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('code_create', body, key),
    codeRotate: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('code_rotate', body, key),
    mine: () => resource.read<{ disputes: import('./types.js').AttributionDispute[] }>('my_attribution'),
    codes: () => resource.read<Promotion[]>('codes'),
    managedCodes: (owner: string) => resource.read<Promotion[]>('managed_codes', owner),
    admin: (scope?: string) => resource.read<AttributionAdmin>('attribution_admin', scope),
    policies: (scope?: string) => resource.read<AttributionAdmin>('attribution_policies', scope),
    policyPublish: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('policy_publish', body, key),
    touchBind: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('touch_bind', body, key),
    attributionDispute: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('attribution_dispute', body, key),
    disputeResolve: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('dispute_resolve', body, key),
  }
}
