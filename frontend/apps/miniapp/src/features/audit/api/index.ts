import { createAudit } from '@crowd/api-contract/audit'
import { transport } from '../../session/api'
export const audit = createAudit(transport)
export type * from '@crowd/api-contract/audit'
