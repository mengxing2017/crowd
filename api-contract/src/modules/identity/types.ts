import type { Receipt } from '../receipts/types.js'
export interface User { id: string; nickname: string; avatar: string | null; version: number; status: string; roles: string[]; scope: string[]; receipts: Receipt[]; restrictions: { scope: string; reason: string; appeal: string }[] }
export interface Login { token: string; user_id: string; expires_in: number }
