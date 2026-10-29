import type { Transport } from '@crowd/api-contract/common'
import { createRequestQueue } from '../../../shared/request-queue'

export const origin = (import.meta.env.VITE_API_ORIGIN || 'http://127.0.0.1:8080').replace(/\/$/, '')
let token = ''
let generation = 0
const listeners = new Set<() => void>()
export const scheduleRequest = createRequestQueue()
export function sessionVersion() { return generation }
export function setToken(value: string) {
  if (value === token) return
  token = value; generation += 1
  if (!value) for (const listener of listeners) listener()
}
export function onSessionExpired(listener: () => void) { listeners.add(listener); return () => { listeners.delete(listener) } }
export function authHeader() { return { Authorization: `Bearer ${token}` } }
export function assertSession(version: number) {
  if (version !== generation) throw new Error('会话已变化，请重新查询当前账号的结果')
}
export function checkStatus(status: number, version: number) {
  assertSession(version)
  if (status === 401) { setToken(''); throw new Error('登录已过期，请重新登录') }
}
export async function request(options: Omit<UniApp.RequestOptions, 'success' | 'fail' | 'complete'>) {
  const version = generation
  return scheduleRequest(() => {
    assertSession(version)
    return new Promise<UniApp.RequestSuccessCallbackResult>((resolve, reject) => {
      uni.request({ ...options, header: { ...options.header, ...authHeader() }, success: result => {
        try { checkStatus(result.statusCode, version); resolve(result) } catch (error) { reject(error) }
      }, fail: () => reject(new Error('网络请求失败，请先查询原提交结果')) })
    })
  })
}
export const transport: Transport = async input => (await request({ url: `${origin}${input.path}`, method: input.method, data: input.body as object | undefined, header: { 'Content-Type': 'application/json', ...(input.key ? { 'Idempotency-Key': input.key } : {}) } })).data
