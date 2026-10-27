import type { Transport } from '../common/types.js'
import type { Login, User } from './types.js'
import type { Rule } from '../governance/types.js'
import { createRequester } from '../common/request.js'
import { createResource } from '../common/resource.js'
import { validateIdentity } from './validation.js'

export function createIdentity(transport: Transport) {
  const call = createRequester(transport, validateIdentity)
  const resource = createResource(transport, validateIdentity)
  return {
    agreements: () => call<Rule>({ method: 'GET', path: '/api/v1/community/agreements' }),
    login: (body: { code: string; agreed: boolean; agreement_id?: string; source?: string; touch_token?: string }) => call<Login>({ method: 'POST', path: '/api/v1/community/login', body }),
    adminLogin: (body: { username: string; password: string }) => call<Login>({ method: 'POST', path: '/api/v1/community/admin/login', body }),
    redeem: (ticket: string) => call<Login>({ method: 'POST', path: '/api/v1/community/session', body: { ticket } }),
    users: (id: string) => resource.read<Pick<User, 'id' | 'version' | 'status' | 'roles' | 'scope' | 'restrictions'>>('users', id),
    eligibility: (id: string) => resource.read<{ version: number; region?: string; qualifications?: string[] }>('eligibility', id),
    receipts: (owner: string) => resource.read<User['receipts']>('receipts', owner),
    phone: (body: Record<string, unknown>, key: string) => resource.command('phone', body, key),
    access: (body: Record<string, unknown>, key: string) => resource.command('access', body, key),
    restriction: (body: Record<string, unknown>, key: string) => resource.command('restriction', body, key),
    me: () => resource.read<User>('me'),
    profile: (body: Record<string, unknown>, key: string) => resource.command('profile', body, key),
    logout: (key: string) => resource.command('logout', {}, key),
    receipt: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('receipt', body, key),
    receiptDisable: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('receipt_disable', body, key),
    receiptReveal: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('receipt_reveal', body, key),
    webTicket: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('web_ticket', body, key),
    eligibilityVerify: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('eligibility_verify', body, key),
  }
}
