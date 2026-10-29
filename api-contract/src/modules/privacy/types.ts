export interface OperationRecord { id: string; version: number; [field: string]: unknown }
export interface ExportDownload { id: string; format: string; content: unknown; expires_at: number }
