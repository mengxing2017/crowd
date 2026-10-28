import { identityCommands } from '../identity/api'
import { receiptsCommands } from '../receipts/api'
import { promotionCommands } from '../promotion/api'
import { governanceCommands } from '../governance/api'
import { supportCommands } from '../support/api'
import { catalogCommands } from '../catalog/api'
import { attributionCommands } from '../attribution/api'
import { tasksCommands } from '../tasks/api'

type Command = (body: Record<string, unknown>, key: string) => Promise<unknown>
const commands: Record<string, Command> = {
  ...identityCommands,
  ...receiptsCommands,
  ...promotionCommands,
  ...governanceCommands,
  ...supportCommands,
  ...catalogCommands,
  ...attributionCommands,
  ...tasksCommands,
}
// Cross-domain retry coordination only; each operation uses its owning client's method.
export async function executeCommand(action: string, body: Record<string, unknown>, key: string): Promise<Record<string, unknown>> {
  if (!Object.prototype.hasOwnProperty.call(commands, action)) throw new Error('当前操作未配置对应领域接口')
  const value = await commands[action](body, key)
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('提交结果不符合契约，请查询原结果后重试')
  return value as Record<string, unknown>
}
