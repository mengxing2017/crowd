import { createTasks } from '@crowd/api-contract/tasks'
import { transport } from '../../session/api'

export const tasks = createTasks(transport)
export type * from '@crowd/api-contract/tasks'
export { taskStatusNames } from '@crowd/api-contract/tasks'
