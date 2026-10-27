import type { Transport } from './types.js'
import { createRequester } from './request.js'

export function createResource(transport: Transport, validate?: Parameters<typeof createRequester>[1]) {
  const call = createRequester(transport, validate)
  return {
    read: <T>(section: string, target?: string) => call<T>({ method: 'GET', path: `/api/v1/community/read/${encodeURIComponent(section)}${target ? `?target=${encodeURIComponent(target)}` : ''}` }),
    command: <T = Record<string, unknown>>(action: string, body: Record<string, unknown>, key: string) => call<T>({ method: 'POST', path: `/api/v1/community/commands/${encodeURIComponent(action)}`, body, key }),
  }
}
