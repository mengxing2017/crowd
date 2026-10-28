export interface SettlementBatchPreview { id: string; count: number; amount_cents: number; results: { id: string; eligible: boolean; issue: string }[]; notice: string }
