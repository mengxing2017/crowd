import { createSettlements } from '@crowd/api-contract/settlements'
import { transport } from '../../session/api'
export const settlements = createSettlements(transport)
export type * from '@crowd/api-contract/settlements'
export const settlementsCommands = {
  settlement_review: settlements.settlementReview,
  settlement_second_review: settlements.settlementSecondReview,
  settlement_batch_preview: settlements.settlementBatchPreview,
  settlement_batch_review: settlements.settlementBatchReview,
  settlement_claim: settlements.settlementClaim,
  settlement_handoff: settlements.settlementHandoff,
  settlement_handoff_accept: settlements.settlementHandoffAccept,
  settlement_exception: settlements.settlementException,
  settlement_unknown: settlements.settlementUnknown,
  settlement_confirm_unpaid: settlements.settlementConfirmUnpaid,
  settlement_register: settlements.settlementRegister,
  settlement_verify: settlements.settlementVerify,
  settlement_correction_propose: settlements.settlementCorrectionPropose,
  settlement_correction_review: settlements.settlementCorrectionReview,
}
