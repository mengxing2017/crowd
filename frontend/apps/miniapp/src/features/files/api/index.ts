import { filePath, uploadPath, decodeUpload } from '@crowd/api-contract/files'
import { origin, authHeader } from '../../session/api'
export type * from '@crowd/api-contract/files'
export function fileUrl(id: string, reason?: string) { return `${origin}${filePath(id, reason)}` }
export function upload(path: string, purpose: string): Promise<{ id: string }> {
  return new Promise((resolve, reject) => {
    const fs = uni.getFileSystemManager()
    fs.readFile({ filePath: path, success: file => {
      uni.request({ url: `${origin}${uploadPath(purpose)}`, method: 'POST', data: file.data, header: { ...authHeader(), 'Content-Type': 'application/octet-stream' }, success: r => { try { resolve(decodeUpload(r.data)) } catch (error) { reject(error) } }, fail: () => reject(new Error('上传失败，请重新选择图片')) })
    }, fail: () => reject(new Error('无法读取图片')) })
  })
}
export function download(url: string): Promise<string> { return new Promise((resolve, reject) => uni.downloadFile({ url, header: authHeader(), success: r => r.statusCode === 200 ? resolve(r.tempFilePath) : reject(new Error('图片不可用，请检查登录与服务配置')), fail: () => reject(new Error('图片下载失败，可复制推广码后重试')) })) }
