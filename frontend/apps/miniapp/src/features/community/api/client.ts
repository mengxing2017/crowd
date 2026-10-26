import { createCommunity } from './index'
const origin = import.meta.env.VITE_API_ORIGIN || 'http://127.0.0.1:8080'
let token = ''
export function setToken(value: string) { token = value }
export function fileUrl(id: string) { return `${origin}/api/v1/community/files/${encodeURIComponent(id)}` }
export function qrUrl(id: string) { return `${origin}/api/v1/community/qr/${encodeURIComponent(id)}` }
export function authHeader() { return { Authorization: `Bearer ${token}` } }
export const community = createCommunity(request => new Promise((resolve, reject) => {
  uni.request({ url: `${origin}${request.path}`, method: request.method, data: request.body as object | undefined, header: { 'Content-Type': 'application/json', ...authHeader(), ...(request.key ? { 'Idempotency-Key': request.key } : {}) }, success: result => { if (result.statusCode === 401) setToken(''); resolve(result.data) }, fail: () => reject(new Error('网络请求失败，请先查询原提交结果')) })
}))
export function upload(path: string, purpose: string): Promise<{ id: string }> {
  return new Promise((resolve, reject) => {
    const fs = uni.getFileSystemManager()
    fs.readFile({ filePath: path, success: file => {
      uni.request({ url: `${origin}/api/v1/community/files/${purpose}`, method: 'POST', data: file.data, header: { ...authHeader(), 'Content-Type': 'application/octet-stream' }, success: r => { const value = r.data as { data?: { id: string }; error?: { message: string; request_id: string } }; if (value.data) resolve(value.data); else reject(new Error(`${value.error?.message || '上传失败'}（${value.error?.request_id || ''}）`)) }, fail: () => reject(new Error('上传失败，请重新选择图片')) })
    }, fail: () => reject(new Error('无法读取图片')) })
  })
}
export function download(url: string): Promise<string> { return new Promise((resolve, reject) => uni.downloadFile({ url, header: authHeader(), success: r => r.statusCode === 200 ? resolve(r.tempFilePath) : reject(new Error('图片不可用，请检查登录与服务配置')), fail: () => reject(new Error('图片下载失败，可复制推广码后重试')) })) }
