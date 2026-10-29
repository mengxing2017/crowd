export interface ApiError { business_code?: string; details?: unknown; code: string; message: string; request_id: string }
export interface Envelope<T> { data: T; request_id: string }
export interface Request { method: 'GET' | 'POST'; path: string; body?: unknown; key?: string }
export type Transport = (request: Request) => Promise<unknown>
