import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import type { WithdrawalRule,WithdrawalApplication } from './types.js'
import { validate } from './validation.js'
export function createWithdrawals(transport: Transport) {
  const resource = createResource(transport, validate)
  return {
    rules: (target?: string) => resource.read<WithdrawalRule | WithdrawalRule[]>('withdrawal_rules', target),
    mine: (target?: string) => resource.read<WithdrawalApplication[]>('my_withdrawals', target),
    queue: (target?: string) => resource.read<WithdrawalApplication[]>('withdrawal_queue', target),
    detail: (target?: string) => resource.read<WithdrawalApplication>('withdrawal_detail', target),
    withdrawalRulePropose: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('withdrawal_rule_propose', body, key),
    withdrawalRuleReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('withdrawal_rule_review', body, key),
    withdrawalPreview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('withdrawal_preview', body, key),
    withdrawalApply: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('withdrawal_apply', body, key),
    withdrawalCancel: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('withdrawal_cancel', body, key),
    withdrawalReceiptReveal: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('withdrawal_receipt_reveal', body, key),
  }
}
