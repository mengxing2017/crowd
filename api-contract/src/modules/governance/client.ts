import type { Transport } from '../common/types.js'
import type { Rule } from './types.js'
import { createResource } from '../common/resource.js'

export function createGovernance(transport: Transport) {
  const resource = createResource(transport)
  return {
    rules: () => resource.read<Rule[]>('rules'),
    audit: <T = Record<string, unknown>[]>(target: string) => resource.read<T>('audit', target),
    cases: () => resource.read<{ id: string; kind: string; status: string }[]>('cases'),
    ruleSave: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('rule_save', body, key),
    rulePublish: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('rule_publish', body, key),
    ruleDisable: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('rule_disable', body, key),
    correction: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('correction', body, key),
    closure: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('closure', body, key),
    merge: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('merge', body, key),
  }
}
