export { createCommunity, BusinessError } from '@crowd/api-contract/community'
export type { User, Rule, Promotion, Receipt, Login, Request, Product, ProductContent, ProductTask, ProductRelease, Category, CatalogAdmin, PublicProduct, PublicCatalog, Claim, AttributionAdmin, AttributionPolicy, AttributionDispute, CatalogNotification } from '@crowd/api-contract/community'
import { createCommunity } from '@crowd/api-contract/community'
import { transport, authHeader } from '../../session/api'
export { hasToken, setToken } from '../../session/api'
export const community = createCommunity(transport)

export async function uploadProductImage(file: File): Promise<string> {
  const response = await fetch('/api/v1/community/files/product_image', { method: 'POST', headers: { ...authHeader(), 'Content-Type': 'application/octet-stream' }, body: file })
  const value = await response.json() as { data?: { id: string }; error?: { message: string; request_id: string } }
  if (!value.data) throw new Error(`${value.error?.message || '上传失败'}（${value.error?.request_id || ''}）`)
  return value.data.id
}

export type { TaskRules, PriceRule, TaskTemplate, EvidenceField, TaskInstance, TaskPage, Evidence, TaskSubmission, VersionedTaskRule } from '@crowd/api-contract/community'
export { taskStatusNames } from '@crowd/api-contract/community'

export async function viewEvidenceImage(id: string, reason: string): Promise<string> {
  const response = await fetch(`/api/v1/community/files/${encodeURIComponent(id)}?reason=${encodeURIComponent(reason)}`, { headers: { ...authHeader() }, cache: 'no-store' })
  if (!response.ok) { const data = await response.json() as { error?: { message: string } }; throw new Error(data.error?.message || '无权查看证据图片') }
  return URL.createObjectURL(await response.blob())
}
