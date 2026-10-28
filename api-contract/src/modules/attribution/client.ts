import type { Transport } from '../common/types.js'
import type { AttributionAdmin } from './types.js'
import { createRequester } from '../common/request.js'
import { createResource } from '../common/resource.js'

export function createAttribution(transport: Transport) {
  const call = createRequester(transport)
  const resource = createResource(transport)
  return {
    touch: (body: { code: string; scene: string }, key: string) => call<{ touch_token: string; product: string | null; task: string | null; notice: string }>({ method: 'POST', path: '/api/v1/catalog/touches', body, key }),
    mine: () => resource.read<{ disputes: import('./types.js').AttributionDispute[] }>('my_attribution'),
    admin: (scope?: string) => resource.read<AttributionAdmin>('attribution_admin', scope),
    policies: (scope?: string) => resource.read<AttributionAdmin>('attribution_policies', scope),
    policyPublish: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('policy_publish', body, key),
    touchBind: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('touch_bind', body, key),
    attributionDispute: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('attribution_dispute', body, key),
    disputeResolve: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('dispute_resolve', body, key),
  }
}
