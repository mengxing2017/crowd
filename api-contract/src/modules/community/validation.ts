import type { Request } from '../common/types.js'
import { validateIdentity } from '../identity/validation.js'
import { validateCatalog } from '../catalog/validation.js'
import { validateTasks } from '../tasks/validation.js'

export function validateResponse(request: Request, value: unknown): void {
  validateIdentity(request, value)
  validateCatalog(request, value)
  validateTasks(request, value)
}
