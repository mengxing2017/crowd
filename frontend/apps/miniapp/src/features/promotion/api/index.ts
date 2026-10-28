import { createPromotion } from '@crowd/api-contract/promotion'
import { transport } from '../../session/api'
import { qrPath } from '@crowd/api-contract/promotion'
import { origin } from '../../session/api'

export const promotion = createPromotion(transport)
export type * from '@crowd/api-contract/promotion'
export { qrPath } from '@crowd/api-contract/promotion'

export const promotionCommands = {
  code_create: promotion.codeCreate,
  code_rotate: promotion.codeRotate,
}

export function qrUrl(code: string) { return `${origin}${qrPath(code)}` }
