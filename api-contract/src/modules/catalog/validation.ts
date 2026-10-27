import type { Request } from '../common/types.js'
import { record } from '../common/validation.js'

export function validateCatalog(request: Request, value: unknown): void {
  if (request.path.startsWith('/api/v1/catalog') && request.method === 'GET') {
    if (!record(value) || !Array.isArray(value.products) || !Array.isArray(value.categories) || !Number.isSafeInteger(value.revision)) throw new Error('产品目录响应不符合契约')
    for (const product of value.products) {
      if (!record(product) || typeof product.id !== 'string' || typeof product.release_id !== 'string' || !record(product.content) || typeof product.content.name !== 'string' || !Array.isArray(product.content.tasks) || !Array.isArray(product.available_tasks)) throw new Error('产品响应不符合契约')
      for (const task of product.content.tasks) if (!record(task) || !Number.isSafeInteger(task.commission_cents)) throw new Error('佣金数字响应不符合契约')
    }
  }

}
