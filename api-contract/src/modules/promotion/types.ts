export interface Promotion { id: string; code: { owner: string; version: number; active: boolean; product: string | null; task: string | null } }
export interface ResolvedPromotion { code: string; version: number; product: string | null; task: string | null; notice: string }
