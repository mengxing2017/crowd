import { createWithdrawals } from '@crowd/api-contract/withdrawals'
import { transport } from '../../session/api'
export const withdrawals = createWithdrawals(transport)
export type * from '@crowd/api-contract/withdrawals'
export { withdrawalStatusNames } from '@crowd/api-contract/withdrawals'
export const withdrawalsCommands = {
  withdrawal_rule_propose: withdrawals.withdrawalRulePropose,
  withdrawal_rule_review: withdrawals.withdrawalRuleReview,
  withdrawal_preview: withdrawals.withdrawalPreview,
  withdrawal_apply: withdrawals.withdrawalApply,
  withdrawal_cancel: withdrawals.withdrawalCancel,
  withdrawal_receipt_reveal: withdrawals.withdrawalReceiptReveal,
}
