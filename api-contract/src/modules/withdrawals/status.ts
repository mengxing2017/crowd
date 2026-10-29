import type { WithdrawalApplication } from './types.js'
export const withdrawalStatusNames: Record<string, string> = { pending_review: '待审核', awaiting_second_review: '待独立复核', ready: '待人工处理', processing: '人工处理中', unknown: '待核实', awaiting_verification: '付款登记待复核', settled: '已登记结算', rejected: '已驳回', cancelled: '已撤销' }

export function withdrawalStatusLabel(application: Pick<WithdrawalApplication, 'status' | 'resolution'>): string {
  return application.resolution === 'returned_unpaid' ? '已退回（确认未付款）' : withdrawalStatusNames[application.status]
}
