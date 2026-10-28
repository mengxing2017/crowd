import type { Request } from '../common/types.js'
import { record } from '../common/validation.js'
const safe = (n: unknown) => typeof n === 'number' && Number.isSafeInteger(n)
export function validate(request: Request, value: unknown): void {
 const section = request.path.split('/').pop()?.split('?')[0]
 if (!record(value) && !Array.isArray(value)) throw new Error('业务响应不符合契约')
 if (section === 'commission_summary') {
  if (!record(value) || value.currency !== 'CNY' || typeof value.consistent !== 'boolean' || !['version','pending_cents','confirmed_cents','available_cents','held_cents','disputed_cents','settled_cents'].every(k => safe(value[k]))) throw new Error('佣金汇总不符合整数分契约')
 }
 if (section === 'commission_entries') {
  if (!record(value) || !Array.isArray(value.items) || !safe(value.total) || !(value.next_cursor === null || safe(value.next_cursor)) || !value.items.every(e => record(e) && safe(e.amount_cents) && typeof e.id === 'string')) throw new Error('流水分页响应无效')
 }
 if (section === 'my_withdrawals' || section === 'withdrawal_queue' || section === 'withdrawal_detail') {
  const rows = Array.isArray(value) ? value : [value]
  if (!rows.every(a => record(a) && typeof a.id === 'string' && safe(a.amount_cents) && safe(a.version) && a.currency === 'CNY' && record(a.receipt_snapshot) && record(a.rule) && Array.isArray(a.events))) throw new Error('提现申请响应无效')
 }
}
