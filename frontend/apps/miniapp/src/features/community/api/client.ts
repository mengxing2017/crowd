import { createCommunity } from './index'
import { origin, authHeader, transport } from '../../session/api'
export { setToken, fileUrl, qrUrl, authHeader } from '../../session/api'
export const community = createCommunity(transport)

export function upload(path: string, purpose: string): Promise<{ id: string }> {
  return new Promise((resolve, reject) => {
    const fs = uni.getFileSystemManager()
    fs.readFile({ filePath: path, success: file => {
      uni.request({ url: `${origin}/api/v1/community/files/${purpose}`, method: 'POST', data: file.data, header: { ...authHeader(), 'Content-Type': 'application/octet-stream' }, success: r => { const value = r.data as { data?: { id: string }; error?: { message: string; request_id: string } }; if (value.data) resolve(value.data); else reject(new Error(`${value.error?.message || '上传失败'}（${value.error?.request_id || ''}）`)) }, fail: () => reject(new Error('上传失败，请重新选择图片')) })
    }, fail: () => reject(new Error('无法读取图片')) })
  })
}
export function download(url: string): Promise<string> { return new Promise((resolve, reject) => uni.downloadFile({ url, header: authHeader(), success: r => r.statusCode === 200 ? resolve(r.tempFilePath) : reject(new Error('图片不可用，请检查登录与服务配置')), fail: () => reject(new Error('图片下载失败，可复制推广码后重试')) })) }
