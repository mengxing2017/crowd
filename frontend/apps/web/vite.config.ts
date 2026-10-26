import { fileURLToPath } from 'node:url'
import { defineConfig, searchForWorkspaceRoot } from 'vite'
import react from '@vitejs/plugin-react'
const contractRoot = fileURLToPath(new URL('../../../api-contract', import.meta.url))
export default defineConfig({
  plugins: [react()],
  // Local file dependencies can retain stale pnpm copies and Vite prebundles.
  // Resolve the contract from its source for both development and production builds.
  resolve: { alias: [{ find: /^@crowd\/api-contract\/([^/]+)$/, replacement: `${contractRoot}/src/modules/$1/index.ts` }] },
  optimizeDeps: { exclude: ['@crowd/api-contract/community', '@crowd/api-contract/system'] },
  server: { host: '127.0.0.1', port: 5173, strictPort: true,
    fs: { allow: [searchForWorkspaceRoot(process.cwd()), contractRoot] },
    proxy: Object.fromEntries(['/api', '/healthz', '/readyz'].map(path => [path, {
      target: process.env.CROWD_API_ORIGIN ?? 'http://127.0.0.1:8080', changeOrigin: true,
    }])) },
})
