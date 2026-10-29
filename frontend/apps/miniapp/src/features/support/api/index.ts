import { createSupport } from '@crowd/api-contract/support'
import { transport } from '../../session/api'
export const support = createSupport(transport)
export type * from '@crowd/api-contract/support'

export const supportCommands = {
  ticket_create: support.ticketCreate,
  ticket_assign: support.ticketAssign,
  ticket_reply: support.ticketReply,
  ticket_close: support.ticketClose,
  ticket_reopen: support.ticketReopen,
  support_policy: support.supportPolicySave,

  correction: support.correction,
  closure: support.closure,
  merge: support.merge,
}
