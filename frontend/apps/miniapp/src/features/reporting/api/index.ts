import { createReporting } from '@crowd/api-contract/reporting'
import { transport } from '../../session/api'
export const reporting = createReporting(transport)
export type * from '@crowd/api-contract/reporting'
export const reportingCommands = {
 report_create: reporting.reportCreate,
 export_request: reporting.exportRequest,
 reconciliation_run: reporting.reconciliationRun,
 difference_assign: reporting.differenceAssign,
 difference_close: reporting.differenceClose,
 voucher_check: reporting.voucherCheck,
}
