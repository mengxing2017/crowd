import { createPlatform } from '@crowd/api-contract/platform'
import { transport } from '../../session/api'
export const platform = createPlatform(transport)
export { BusinessError } from '@crowd/api-contract/common'
export { executeCommand } from '../commands'
