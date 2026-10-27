import type { Request } from '../common/types.js'
import { record } from '../common/validation.js'

export function validateTasks(request: Request, value: unknown): void {
  if (request.path.startsWith('/api/v1/community/read/my_tasks') || request.path.startsWith('/api/v1/community/read/task_queue')) {
    if (!record(value) || !Array.isArray(value.items) || !(value.next_cursor === null || typeof value.next_cursor === 'string')) throw new Error('任务列表响应不符合契约')
    for (const item of value.items) if (!record(item) || typeof item.id !== 'string' || !Number.isSafeInteger(item.version) || typeof item.status !== 'string' || !record(item.template) || !Number.isSafeInteger(item.reserved_cents)) throw new Error('任务实例响应不符合契约')
  }
  if (request.path.startsWith('/api/v1/community/read/task_rules')) {
    if (!record(value) || !Array.isArray(value.prices) || !Array.isArray(value.templates) || !Number.isSafeInteger(value.server_time)) throw new Error('任务规则响应不符合契约')
    for (const rule of [...value.prices, ...value.templates]) if (!record(rule) || typeof rule.id !== 'string' || !Number.isSafeInteger(rule.version) || typeof rule.status !== 'string' || !record(rule.content)) throw new Error('任务规则版本不符合契约')
    for (const rule of value.prices) { const content = (rule as { content: Record<string, unknown> }).content; if (content.currency !== 'CNY' || (content.kind === 'fixed' && !Number.isSafeInteger(content.amount_cents)) || (content.kind === 'proportional' && (!Number.isSafeInteger(content.numerator) || !Number.isSafeInteger(content.denominator)))) throw new Error('佣金定价响应不符合契约') }
  }
}
