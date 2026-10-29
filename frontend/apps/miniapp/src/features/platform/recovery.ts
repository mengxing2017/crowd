export interface PendingCommand { action: string; key: string; owner: string; body?: Record<string, unknown> }
export interface RecoveredCommand { action: string; result: Record<string, unknown> }
const storageKey = 'crowd.command-reference.v1'
function references(): Record<string, PendingCommand> {
  try { const value: unknown = uni.getStorageSync(storageKey); return value && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, PendingCommand> : {} } catch { return {} }
}
export function restoreCommand(owner: string): PendingCommand | null {
  const item = references()[owner]
  if (!item || item.owner !== owner || typeof item.action !== 'string' || typeof item.key !== 'string') return null
  return { action: item.action, owner, key: item.key }
}
export function rememberCommand(item: PendingCommand) {
  // No token, account, receipt, evidence or request payload is persisted.
  uni.setStorageSync(storageKey, { ...references(), [item.owner]: { action: item.action, owner: item.owner, key: item.key } })
}
export function forgetCommand(item: PendingCommand) {
  const items = references()
  if (items[item.owner]?.key === item.key) { delete items[item.owner]; try { uni.setStorageSync(storageKey, items) } catch { /* A stale reference can safely be queried again; never mask a committed result. */ } }
}
const listeners = new Set<(value: RecoveredCommand) => void>()
export function onCommandRecovered(listener: (value: RecoveredCommand) => void) { listeners.add(listener); return () => { listeners.delete(listener) } }
export function publishRecovered(value: RecoveredCommand) { for (const listener of listeners) listener(value) }
