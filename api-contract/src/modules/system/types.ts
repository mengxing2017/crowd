export interface SystemInfo { application: string; version: string; stage: string; capabilities?: string[]; read_compatibility?: string }
export interface SystemDocument { data: { type: 'systems'; id: 'crowd'; attributes: SystemInfo } }
export type Transport = (path: string) => Promise<unknown>
