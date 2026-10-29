import type { Transport } from '../common/types.js'
import type { Login, User } from './types.js'
import { createRequester } from '../common/request.js'
import { createResource } from '../common/resource.js'
import { validateIdentity } from './validation.js'

export function createIdentity(transport: Transport) {
  const call = createRequester(transport, validateIdentity)
  const resource = createResource(transport, validateIdentity)
  return {
 temporaryGrantRevoke: (body: Record<string, unknown>, key: string) => resource.command('temporary_grant_revoke', body, key),
 temporaryGrantReview: (body: Record<string, unknown>, key: string) => resource.command('temporary_grant_review', body, key),
 temporaryGrantRequest: (body: Record<string, unknown>, key: string) => resource.command('temporary_grant_request', body, key),
 securityPolicyPublish: (body: Record<string, unknown>, key: string) => resource.command('security_policy_publish', body, key),
    reauthenticate: (password: string) => call<{ verified: boolean; policy_version: number; expires_in: number }>({ method: 'POST', path: '/api/v1/identity/reauthenticate', body: { password } }),
    security: () => resource.read<{ policy: Record<string, unknown> | null; grants: Record<string, unknown>[] }>('security'),
    login: (body: { code: string; agreed: boolean; agreement_id?: string; source?: string; touch_token?: string }) => call<Login>({ method: 'POST', path: '/api/v1/community/login', body }),
    adminLogin: (body: { username: string; password: string }) => call<Login>({ method: 'POST', path: '/api/v1/community/admin/login', body }),
    redeem: (ticket: string) => call<Login>({ method: 'POST', path: '/api/v1/community/session', body: { ticket } }),
    users: (id: string) => resource.read<Pick<User, 'id' | 'version' | 'status' | 'roles' | 'scope' | 'restrictions'>>('users', id),
    phone: (body: Record<string, unknown>, key: string) => resource.command('phone', body, key),
    access: (body: Record<string, unknown>, key: string) => resource.command('access', body, key),
    restriction: (body: Record<string, unknown>, key: string) => resource.command('restriction', body, key),
    me: () => resource.read<User>('me'),
    profile: (body: Record<string, unknown>, key: string) => resource.command('profile', body, key),
    logout: (key: string) => resource.command('logout', {}, key),
    webTicket: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('web_ticket', body, key),
  }
}
