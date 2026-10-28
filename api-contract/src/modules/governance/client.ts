import type { Transport } from '../common/types.js'
import type { Rule } from './types.js'
import { createRequester } from '../common/request.js'
import { createResource } from '../common/resource.js'

export function createGovernance(transport: Transport) {
  const resource = createResource(transport)
  const call = createRequester(transport)
  return {
    agreements: () => call<Rule>({ method: 'GET', path: '/api/v1/community/agreements' }),
    rules: () => resource.read<Rule[]>('rules'),
    ruleSave: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('rule_save', body, key),
    rulePublish: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('rule_publish', body, key),
    ruleDisable: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('rule_disable', body, key),
  }
}
