import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import type { CommissionDispute,CommissionPage,CommissionProposal,CommissionSummary,CommissionSource } from './types.js'
import { validate } from './validation.js'
export function createCommissions(transport: Transport) {
  const resource = createResource(transport, validate)
  return {
 adjustmentLocate: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('adjustment_locate', body, key),

    adjustments: (target?: string) => resource.read<Record<string, unknown>[]>('adjustments', target),
    adjustmentPropose: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('adjustment_propose', body, key),
    adjustmentReview: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('adjustment_review', body, key),
    adjustmentBatchPreview: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('adjustment_batch_preview', body, key),
    adjustmentBatchReview: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('adjustment_batch_review', body, key),


    source: (owner: string, source: string) => resource.read<CommissionSource>('commission_source', `${owner}|${source}`),
    summary: (target?: string) => resource.read<CommissionSummary>('commission_summary', target),
    entries: (target?: string) => resource.read<CommissionPage>('commission_entries', target),
    proposals: (target?: string) => resource.read<CommissionProposal[]>('commission_proposals', target),
    disputes: (target?: string) => resource.read<CommissionDispute[]>('commission_disputes', target),
    commissionAdjustPropose: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('commission_adjust_propose', body, key),
    commissionAdjustReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('commission_adjust_review', body, key),
    commissionDispute: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('commission_dispute', body, key),
    commissionDisputeResolve: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('commission_dispute_resolve', body, key),
  }
}
