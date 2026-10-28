import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import type { AuditRecord } from './types.js'
import { validateAudit } from './validation.js'

export function createAudit(transport: Transport) {
  const resource = createResource(transport, validateAudit)
  return {
    records: <T = AuditRecord[]>(target: string) => resource.read<T>('audit', target),
  }
}
