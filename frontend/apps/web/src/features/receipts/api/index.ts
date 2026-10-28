import { createReceipts } from '@crowd/api-contract/receipts'
import { transport } from '../../session/api'
export const receipts = createReceipts(transport)
export type * from '@crowd/api-contract/receipts'

export const receiptsCommands = {
  receipt: receipts.receipt,
  receipt_disable: receipts.receiptDisable,
  receipt_reveal: receipts.receiptReveal,
}
