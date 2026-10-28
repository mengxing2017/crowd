import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'

export function createPlatform(transport: Transport) {
  const resource = createResource(transport)
  return {
    commandResult: <T = Record<string, unknown>>(key: string) => resource.read<T>('command_result', key),
  }
}
