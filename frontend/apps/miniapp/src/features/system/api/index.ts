import { getSystem } from '@crowd/api-contract/system'
import { transport } from '../../session/api'
export const system = () => getSystem(path => transport({ method: 'GET', path }))
