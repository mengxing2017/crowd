import { createCatalog } from '@crowd/api-contract/catalog'
import { transport } from '../../session/api'

export const catalog = createCatalog(transport)
export type * from '@crowd/api-contract/catalog'
