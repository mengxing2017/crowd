import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import type { SupportCase } from './types.js'
import { validateSupport } from './validation.js'

export function createSupport(transport: Transport) {
  const resource = createResource(transport, validateSupport)
  return {
    tickets: (target?: string) => resource.read<Record<string, unknown>[]>('tickets', target),
    supportStatistics: (target?: string) => resource.read<Record<string, unknown>>('support_statistics', target),
    supportPolicy: (target?: string) => resource.read<Record<string, unknown>>('support_policy', target),
    ticketCreate: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('ticket_create', body, key),
    ticketAssign: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('ticket_assign', body, key),
    ticketReply: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('ticket_reply', body, key),
    ticketClose: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('ticket_close', body, key),
    ticketReopen: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('ticket_reopen', body, key),
    supportPolicySave: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('support_policy', body, key),


    cases: () => resource.read<SupportCase[]>('cases'),
    correction: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('correction', body, key),
    closure: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('closure', body, key),
    merge: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('merge', body, key),
  }
}
