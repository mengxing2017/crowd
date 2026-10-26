import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
export default defineConfig({
  plugins: [react()],
  server: { host: '127.0.0.1', port: 5173, strictPort: true,
    proxy: Object.fromEntries(['/api', '/healthz', '/readyz'].map(path => [path, {
      target: process.env.CROWD_API_ORIGIN ?? 'http://127.0.0.1:8080', changeOrigin: true,
    }])) },
})
