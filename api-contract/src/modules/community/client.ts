import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import { createIdentity } from '../identity/client.js'
import { createCatalog } from '../catalog/client.js'
import { createAttribution } from '../attribution/client.js'
import { createGovernance } from '../governance/client.js'
import { createPromotion } from '../promotion/client.js'
import { validateResponse } from './validation.js'

/** @deprecated Use the independent domain clients for new features. */
export function createCommunity(transport: Transport) {
  const identity = createIdentity(transport)
  const catalog = createCatalog(transport)
  const attribution = createAttribution(transport)
  return {
    ...createResource(transport, validateResponse),
    agreements: createGovernance(transport).agreements,
    login: identity.login,
    adminLogin: identity.adminLogin,
    redeem: identity.redeem,
    catalog: catalog.catalog,
    touch: attribution.touch,
    resolve: createPromotion(transport).resolve,
  }
}
