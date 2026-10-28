import type { UploadedFile } from './types.js'
import { record } from '../common/validation.js'
import { unwrapEnvelope } from '../common/request.js'
export function decodeUpload(raw: unknown): UploadedFile {
  const value = unwrapEnvelope<unknown>(raw)
  if (!record(value) || typeof value.id !== 'string' || typeof value.purpose !== 'string' || value.official_verified !== false) throw new Error('文件上传响应不符合契约')
  return value as unknown as UploadedFile
}
