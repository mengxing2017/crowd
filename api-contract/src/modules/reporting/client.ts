import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import { validate } from './validation.js'
import type { OperationRecord, ExportDownload } from './types.js'
export function createReporting(transport: Transport) {
 const resource = createResource(transport, validate)
 return {
 reportCreate: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('report_create', body, key),
 exportRequest: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('export_request', body, key),
 reconciliationRun: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('reconciliation_run', body, key),
 differenceAssign: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('difference_assign', body, key),
 differenceClose: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('difference_close', body, key),
 voucherCheck: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('voucher_check', body, key),
 reports: (target?: string) => resource.read<OperationRecord[]>('reports', target),
 reportDetail: (target?: string) => resource.read<OperationRecord>('report_detail', target),
 exportJobs: (target?: string) => resource.read<OperationRecord[]>('export_jobs', target),
 exportDownload: (target?: string) => resource.read<ExportDownload>('export_download', target),
 differences: (target?: string) => resource.read<OperationRecord[]>('differences', target),
 }
}
