import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import type { Receipt } from './types.js'
import { validateReceipts } from './validation.js'

export function createReceipts(transport: Transport) {
  const resource = createResource(transport, validateReceipts)
  return {
    receipts: (owner: string) => resource.read<Receipt[]>('receipts', owner),
    receipt: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('receipt', body, key),
    receiptCleanup: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('receipt_cleanup', body, key),
    receiptDisable: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('receipt_disable', body, key),
    receiptReveal: <T = Record<string, unknown>>(body: Record<string, unknown>, key: string) => resource.command<T>('receipt_reveal', body, key),
  }
}
