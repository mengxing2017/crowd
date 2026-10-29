import { filePath, uploadPath, decodeUpload } from '@crowd/api-contract/files'
import { origin, authHeader, request, scheduleRequest, sessionVersion, assertSession, checkStatus } from '../../session/api'
export type * from '@crowd/api-contract/files'
export function fileUrl(id: string, reason?: string) { return `${origin}${filePath(id, reason)}` }
export function removeTempFile(path: string) { if (path) uni.getFileSystemManager().unlink({ filePath: path, fail: () => undefined }) }
export async function chooseImagePath(original = false): Promise<string> {
  try {
    if (!uni.canIUse('chooseMedia')) return await new Promise<string>((resolve, reject) => uni.chooseImage({ count: 1, sizeType: [original ? 'original' : 'compressed'], sourceType: ['album', 'camera'], success: choice => resolve(choice.tempFilePaths[0]), fail: reject }))
    return await new Promise<string>((resolve, reject) => uni.chooseMedia({ count: 1, mediaType: ['image'], sizeType: [original ? 'original' : 'compressed'], sourceType: ['album', 'camera'], success: choice => resolve(choice.tempFiles[0].tempFilePath), fail: reject }))
  } catch (error) {
    const msg = (error as { errMsg?: string }).errMsg || ''
    throw new Error(msg.includes('cancel') ? '已取消选择图片，原内容保留' : '图片选择未获授权或失败，可重试；不影响基本业务查询')
  }
}
export async function upload(path: string, purpose: string): Promise<{ id: string }> {
  const version = sessionVersion()
  const fs = uni.getFileSystemManager()
  const info = await new Promise<UniApp.GetFileInfoSuccessCallbackResult>((resolve, reject) => fs.getFileInfo({ filePath: path, success: resolve, fail: () => reject(new Error('无法读取图片信息')) }))
  // Match the existing server engineering ceiling; the server remains authoritative.
  if (info.size > 5 * 1024 * 1024) throw new Error('图片超过当前服务 5 MiB 处理上限，请压缩后重试')
  const image = await uni.getImageInfo({ src: path })
  if (!['png', 'jpeg', 'jpg'].includes((image.type || '').toLowerCase())) throw new Error('当前服务仅支持 PNG/JPEG 图片，请转换格式后重试')
  if (image.width > 4096 || image.height > 4096) throw new Error('图片尺寸超过当前服务 4096 像素处理上限，请缩小后重试')
  const data = await new Promise<string | ArrayBuffer>((resolve, reject) => fs.readFile({ filePath: path, success: file => resolve(file.data), fail: () => reject(new Error('无法读取图片')) }))
  assertSession(version)
  const result = await request({ url: `${origin}${uploadPath(purpose)}`, method: 'POST', data, header: { 'Content-Type': 'application/octet-stream' } })
  return decodeUpload(result.data)
}
export async function download(url: string): Promise<string> {
  const version = sessionVersion()
  return scheduleRequest(() => {
    assertSession(version)
    return new Promise<string>((resolve, reject) => uni.downloadFile({ url, header: authHeader(), success: r => {
      try { checkStatus(r.statusCode, version); if (r.statusCode !== 200) throw new Error('图片不可用，请检查登录与服务配置'); resolve(r.tempFilePath) }
      catch (error) { removeTempFile(r.tempFilePath); reject(error) }
    }, fail: () => reject(new Error('图片下载失败，请重试或返回原业务查询')) }))
  })
}
