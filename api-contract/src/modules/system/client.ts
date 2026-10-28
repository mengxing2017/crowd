import type { SystemDocument, Transport } from './types.js'
// 平台自行注入 transport；协议包不依赖 DOM、fetch 或微信运行时。
export async function getSystem(transport: Transport): Promise<SystemDocument> {
  const value = await transport('/api/v1/system')
  const doc = value as SystemDocument | null
  const data = doc?.data
  const info = data?.attributes
  if (data?.type !== 'systems' || data.id !== 'crowd' ||
      typeof info?.application !== 'string' || typeof info.version !== 'string' || typeof info.stage !== 'string') {
    throw new Error('Invalid system response')
  }
  return doc as SystemDocument
}
