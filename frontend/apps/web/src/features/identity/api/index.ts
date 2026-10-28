import { createIdentity } from '@crowd/api-contract/identity'
import { transport } from '../../session/api'

export const identity = createIdentity(transport)
export type * from '@crowd/api-contract/identity'

export const identityCommands = {
  phone: identity.phone,
  access: identity.access,
  restriction: identity.restriction,
  profile: identity.profile,
  logout: (_body: Record<string, unknown>, key: string) => identity.logout(key),
  web_ticket: identity.webTicket,
}
