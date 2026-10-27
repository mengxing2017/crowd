import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'
import { defineConfig } from 'vite'
// DCloud publishes this plugin as CommonJS with a default export.
process.env.CI ??= 'true'
const uni = createRequire(import.meta.url)('@dcloudio/vite-plugin-uni').default
const contractRoot = fileURLToPath(new URL('../../../api-contract', import.meta.url))
export default defineConfig({
  plugins: [uni()],
  resolve: { alias: [{ find: /^@crowd\/api-contract\/([^/]+)$/, replacement: `${contractRoot}/src/modules/$1/index.ts` }] },
})
