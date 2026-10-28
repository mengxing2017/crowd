export interface AttributionPolicy { id: string; version: number; window_seconds: number; strategy: 'first_touch' | 'last_touch' }
export interface AttributionDispute { id: string; version: number; owner: string; claim_id: string; reason: string; evidence: string; status: string; decision_reason?: string }
export interface AttributionAdmin { policies: AttributionPolicy[]; touch_count: number; disputes: AttributionDispute[] }
