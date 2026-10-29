import { onHide, onShow } from '@dcloudio/uni-app'
import { onUnmounted } from 'vue'
import { createAsyncScope } from './async-scope'
export function usePrivateScope(clear: () => void) {
  const scope = createAsyncScope()
  const reset = () => { scope.invalidate(); clear() }
  onHide(() => { scope.hide(); clear() })
  onShow(scope.show)
  onUnmounted(() => { scope.hide(); clear() })
  return { ...scope, clear: reset }
}
