import type { Transport } from '../common/types.js'
import type { Claim, TaskInstance, TaskPage, TaskRules, TaskEligibility } from './types.js'
import { createResource } from '../common/resource.js'
import { validateTasks } from './validation.js'

export function createTasks(transport: Transport) {
  const resource = createResource(transport, validateTasks)
  return {
    eligibility: (id: string) => resource.read<TaskEligibility>('eligibility', id),
    eligibilityVerify: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('eligibility_verify', body, key),
    rules: () => resource.read<TaskRules>('task_rules'),
    mine: (filter?: string) => resource.read<TaskPage>('my_tasks', filter),
    queue: (filter?: string) => resource.read<TaskPage>('task_queue', filter),
    detail: (id: string) => resource.read<TaskInstance>('task_detail', id),
    claims: () => resource.read<Claim[]>('my_claims'),
    claim: (body: Record<string, unknown>, key: string) => resource.command<Claim>('task_claim', body, key),
    priceSave: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('price_save', body, key),
    priceReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('price_review', body, key),
    templateSave: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('template_save', body, key),
    templateReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('template_review', body, key),
    templateDisable: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('template_disable', body, key),
    taskStart: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('task_start', body, key),
    taskStep: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('task_step', body, key),
    taskAbandon: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('task_abandon', body, key),
    taskPause: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('task_pause', body, key),
    taskResume: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('task_resume', body, key),
    taskExpire: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('task_expire', body, key),
    evidenceDiscard: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('evidence_discard', body, key),
    evidenceSave: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('evidence_save', body, key),
    evidencePreview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('evidence_preview', body, key),
    evidenceSubmit: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('evidence_submit', body, key),
    evidenceWithdraw: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('evidence_withdraw', body, key),
    reviewClaim: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('review_claim', body, key),
    reviewDecide: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('review_decide', body, key),
    reviewVerify: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('review_verify', body, key),
    reviewBatchPreview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('review_batch_preview', body, key),
    reviewBatch: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('review_batch', body, key),
    taskRepairPropose: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('task_repair_propose', body, key),
    taskRepairReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('task_repair_review', body, key),
    reviewRevokePropose: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('review_revoke_propose', body, key),
    reviewRevokeReview: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('review_revoke_review', body, key),
    qualityInspect: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('quality_inspect', body, key),
    evidenceReveal: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('evidence_reveal', body, key),
  }
}
