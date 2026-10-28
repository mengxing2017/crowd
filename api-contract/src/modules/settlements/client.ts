import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import { validate } from './validation.js'
export function createSettlements(transport: Transport) {
  const resource = createResource(transport, validate)
  return {
    settlementReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_review', body, key),
    settlementSecondReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_second_review', body, key),
    settlementBatchPreview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_batch_preview', body, key),
    settlementBatchReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_batch_review', body, key),
    settlementClaim: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_claim', body, key),
    settlementHandoff: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_handoff', body, key),
    settlementHandoffAccept: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_handoff_accept', body, key),
    settlementUnknown: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_unknown', body, key),
    settlementConfirmUnpaid: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_confirm_unpaid', body, key),
    settlementRegister: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_register', body, key),
    settlementVerify: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_verify', body, key),
    settlementCorrectionPropose: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_correction_propose', body, key),
    settlementCorrectionReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('settlement_correction_review', body, key),
  }
}
