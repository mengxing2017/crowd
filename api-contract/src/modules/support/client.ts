import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import type { SupportCase } from './types.js'
import { validateSupport } from './validation.js'

export function createSupport(transport: Transport) {
  const resource = createResource(transport, validateSupport)
  return {
    cases: () => resource.read<SupportCase[]>('cases'),
    correction: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('correction', body, key),
    closure: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('closure', body, key),
    merge: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('merge', body, key),
  }
}
