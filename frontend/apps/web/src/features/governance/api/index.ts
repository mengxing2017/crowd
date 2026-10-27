import { createGovernance } from '@crowd/api-contract/governance'
import { transport } from '../../session/api'

export const governance = createGovernance(transport)
export type * from '@crowd/api-contract/governance'
