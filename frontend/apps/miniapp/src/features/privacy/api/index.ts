import { createPrivacy } from '@crowd/api-contract/privacy'
import { transport } from '../../session/api'
export const privacy = createPrivacy(transport)
export type * from '@crowd/api-contract/privacy'
export const privacyCommands = {
 privacy_policy_publish: privacy.privacyPolicyPublish,
 privacy_copy_request: privacy.privacyCopyRequest,
 privacy_close_request: privacy.privacyCloseRequest,
 privacy_close_review: privacy.privacyCloseReview,
 privacy_repair_preview: privacy.privacyRepairPreview,
 privacy_repair_review: privacy.privacyRepairReview,
}
