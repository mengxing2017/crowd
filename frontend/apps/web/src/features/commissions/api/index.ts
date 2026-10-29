import { createCommissions } from '@crowd/api-contract/commissions'
import { transport } from '../../session/api'
export const commissions = createCommissions(transport)
export type * from '@crowd/api-contract/commissions'
export const commissionsCommands = {
 adjustment_locate: commissions.adjustmentLocate,
  adjustment_propose: commissions.adjustmentPropose,
  adjustment_review: commissions.adjustmentReview,
  adjustment_batch_preview: commissions.adjustmentBatchPreview,
  adjustment_batch_review: commissions.adjustmentBatchReview,

  commission_adjust_propose: commissions.commissionAdjustPropose,
  commission_adjust_review: commissions.commissionAdjustReview,
  commission_dispute: commissions.commissionDispute,
  commission_dispute_resolve: commissions.commissionDisputeResolve,
}
