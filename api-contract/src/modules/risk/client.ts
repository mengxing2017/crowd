import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import { validate } from './validation.js'
import type { RiskRecord } from './types.js'
export function createRisk(transport: Transport) {
 const resource = createResource(transport, validate)
 return {
 riskCaseOpen: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('risk_case_open', body, key),

    riskRules: (target?: string) => resource.read<RiskRecord[]>('risk_rules', target),
    riskCases: (target?: string) => resource.read<RiskRecord[]>('risk_cases', target),
    myRisk: (target?: string) => resource.read<RiskRecord[]>('my_risk', target),
    riskRulePublish: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('risk_rule_publish', body, key),
    riskScan: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('risk_scan', body, key),
    riskCaseDecide: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('risk_case_decide', body, key),
 }
}
