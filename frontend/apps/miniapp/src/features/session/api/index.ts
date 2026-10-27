import type { Transport } from '@crowd/api-contract/common'

export const origin = import.meta.env.VITE_API_ORIGIN || 'http://127.0.0.1:8080'
let token = ''
export function setToken(value: string) { token = value }
export function fileUrl(id: string) { return `${origin}/api/v1/community/files/${encodeURIComponent(id)}` }
export function qrUrl(id: string) { return `${origin}/api/v1/community/qr/${encodeURIComponent(id)}` }
export function authHeader() { return { Authorization: `Bearer ${token}` } }
export const transport: Transport = request => new Promise((resolve, reject) => {
  uni.request({ url: `${origin}${request.path}`, method: request.method, data: request.body as object | undefined, header: { 'Content-Type': 'application/json', ...authHeader(), ...(request.key ? { 'Idempotency-Key': request.key } : {}) }, success: result => { if (result.statusCode === 401) setToken(''); resolve(result.data) }, fail: () => reject(new Error('网络请求失败，请先查询原提交结果')) })
})