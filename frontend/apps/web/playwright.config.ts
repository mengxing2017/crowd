import { defineConfig, devices } from '@playwright/test'
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  retries: 0,
  timeout: 30000,
  expect: { timeout: 10000 },
  outputDir: './test-results',
  reporter: 'list',
  use: { baseURL: 'http://127.0.0.1:5178', trace: 'retain-on-failure' },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: { command: process.env.CROWD_WEB_E2E_PREVIEW === '1' ? 'pnpm exec vite preview --host 127.0.0.1 --port 5178 --strictPort' : 'pnpm exec vite --host 127.0.0.1 --port 5178', url: 'http://127.0.0.1:5178', reuseExistingServer: false },
})
