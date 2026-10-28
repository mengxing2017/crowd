import { createSupport } from '@crowd/api-contract/support'
import { transport } from '../../session/api'
export const support = createSupport(transport)
export type * from '@crowd/api-contract/support'

export const supportCommands = {
  correction: support.correction,
  closure: support.closure,
  merge: support.merge,
}
