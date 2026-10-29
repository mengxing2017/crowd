import { createTasks } from '@crowd/api-contract/tasks'
import { transport } from '../../session/api'

export const tasks = createTasks(transport)
export type * from '@crowd/api-contract/tasks'
export { taskStatusNames } from '@crowd/api-contract/tasks'

export const tasksCommands = {
 task_source_configure: tasks.taskSourceConfigure,
  eligibility_verify: tasks.eligibilityVerify,
  task_claim: tasks.claim,
  price_save: tasks.priceSave,
  price_review: tasks.priceReview,
  template_save: tasks.templateSave,
  template_review: tasks.templateReview,
  template_disable: tasks.templateDisable,
  task_start: tasks.taskStart,
  task_step: tasks.taskStep,
  task_abandon: tasks.taskAbandon,
  task_pause: tasks.taskPause,
  task_resume: tasks.taskResume,
  task_expire: tasks.taskExpire,
  evidence_discard: tasks.evidenceDiscard,
  evidence_save: tasks.evidenceSave,
  evidence_preview: tasks.evidencePreview,
  evidence_submit: tasks.evidenceSubmit,
  evidence_withdraw: tasks.evidenceWithdraw,
  review_claim: tasks.reviewClaim,
  review_decide: tasks.reviewDecide,
  review_verify: tasks.reviewVerify,
  review_batch_preview: tasks.reviewBatchPreview,
  review_batch: tasks.reviewBatch,
  task_repair_propose: tasks.taskRepairPropose,
  task_repair_review: tasks.taskRepairReview,
  review_revoke_propose: tasks.reviewRevokePropose,
  review_revoke_review: tasks.reviewRevokeReview,
  quality_inspect: tasks.qualityInspect,
  evidence_reveal: tasks.evidenceReveal,
}
