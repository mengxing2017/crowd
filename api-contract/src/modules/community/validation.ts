import type { Request } from '../common/types.js'
import { validateIdentity } from '../identity/validation.js'
import { validateCatalog } from '../catalog/validation.js'
import { validateTasks } from '../tasks/validation.js'
import { validateReceipts } from '../receipts/validation.js'
import { validatePromotion } from '../promotion/validation.js'
import { validateAudit } from '../audit/validation.js'
import { validateSupport } from '../support/validation.js'
export function validateResponse(request: Request, value: unknown): void {
  validateIdentity(request, value)
  validateCatalog(request, value)
  validateTasks(request, value)
  validateReceipts(request, value)
  validatePromotion(request, value)
  validateAudit(request, value)
  validateSupport(request, value)
}
