import { createGovernance } from '@crowd/api-contract/governance'
import { transport } from '../../session/api'

export const governance = createGovernance(transport)
export type * from '@crowd/api-contract/governance'

export const governanceCommands = {
  rule_save: governance.ruleSave,
  rule_publish: governance.rulePublish,
  rule_disable: governance.ruleDisable,
}
