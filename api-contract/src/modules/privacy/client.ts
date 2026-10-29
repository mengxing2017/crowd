import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import { validate } from './validation.js'
import type { OperationRecord } from './types.js'
export function createPrivacy(transport: Transport) {
 const resource = createResource(transport, validate)
 return {
 privacyPolicyPublish: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('privacy_policy_publish', body, key),
 privacyCopyRequest: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('privacy_copy_request', body, key),
 privacyCloseRequest: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('privacy_close_request', body, key),
 privacyCloseReview: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('privacy_close_review', body, key),
 privacyRepairPreview: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('privacy_repair_preview', body, key),
 privacyRepairReview: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('privacy_repair_review', body, key),
 privacyPolicy: (target?: string) => resource.read<OperationRecord | null>('privacy_policy', target),
 privacyRequests: (target?: string) => resource.read<OperationRecord[]>('privacy_requests', target),
 privacyCopy: (target?: string) => resource.read<OperationRecord>('privacy_copy', target),
 privacyRepairs: (target?: string) => resource.read<OperationRecord[]>('privacy_repairs', target),
 }
}
