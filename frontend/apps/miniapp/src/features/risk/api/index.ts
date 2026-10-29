import { createRisk } from '@crowd/api-contract/risk'
import { transport } from '../../session/api'
export const risk = createRisk(transport)
export type * from '@crowd/api-contract/risk'
export const riskCommands = {
 risk_case_open: risk.riskCaseOpen,
 risk_rule_publish: risk.riskRulePublish,
 risk_scan: risk.riskScan,
 risk_case_decide: risk.riskCaseDecide,
}
