import { onUnmounted } from 'vue'
import { onSessionExpired } from '../session/api'
import { usePrivateScope } from '../../shared/use-private-scope'
import { download, removeTempFile } from './api'
export function usePrivateFiles(clear: () => void) {
  const paths = new Set<string>()
  const scope = usePrivateScope(() => {
    for (const path of paths) removeTempFile(path)
    paths.clear(); clear()
  })
  const stop = onSessionExpired(scope.clear)
  onUnmounted(stop)
  return { ...scope, async download(url: string, ticket = scope.capture()) {
    const path = await download(url)
    if (!scope.current(ticket)) { removeTempFile(path); return undefined }
    paths.add(path)
    return path
  } }
}
