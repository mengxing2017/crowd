import { createIdentity } from '@crowd/api-contract/identity'
import { transport } from '../../session/api'

export const identity = createIdentity(transport)
export type * from '@crowd/api-contract/identity'
