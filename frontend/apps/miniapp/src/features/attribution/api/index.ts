import { createAttribution } from '@crowd/api-contract/attribution'
import { transport } from '../../session/api'

export const attribution = createAttribution(transport)
export type * from '@crowd/api-contract/attribution'

export const attributionCommands = {
  policy_publish: attribution.policyPublish,
  touch_bind: attribution.touchBind,
  attribution_dispute: attribution.attributionDispute,
  dispute_resolve: attribution.disputeResolve,
}
