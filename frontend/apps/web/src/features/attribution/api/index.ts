import { createAttribution } from '@crowd/api-contract/attribution'
import { transport } from '../../session/api'

export const attribution = createAttribution(transport)
export type * from '@crowd/api-contract/attribution'
