import { filePath, uploadPath, decodeUpload } from '@crowd/api-contract/files'
import { authHeader } from '../../session/api'
export type * from '@crowd/api-contract/files'
export async function uploadProductImage(file: File): Promise<string> {
  const response = await fetch(uploadPath('product_image'), { method: 'POST', headers: { ...authHeader(), 'Content-Type': 'application/octet-stream' }, body: file })
  return decodeUpload(await response.json()).id
}
export async function viewPrivateImage(id: string, reason?: string): Promise<string> {
  const response = await fetch(filePath(id, reason), { headers: authHeader(), cache: 'no-store' })
  if (!response.ok) { const data = await response.json() as { error?: { message: string } }; throw new Error(data.error?.message || '无权查看私有图片') }
  return URL.createObjectURL(await response.blob())
}

export async function uploadSettlementVoucher(file: File): Promise<string> {
 const response = await fetch(uploadPath('settlement_voucher'), { method: 'POST', headers: { ...authHeader(), 'Content-Type': 'application/octet-stream' }, body: file })
 return decodeUpload(await response.json()).id
}

export async function uploadSupportAttachment(file: File): Promise<string> {
 const response = await fetch(uploadPath('support_attachment'), { method: 'POST', headers: { ...authHeader(), 'Content-Type': 'application/octet-stream' }, body: file })
 return decodeUpload(await response.json()).id
}
