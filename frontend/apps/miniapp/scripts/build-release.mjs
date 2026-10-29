import { readFileSync, writeFileSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import { loadEnv } from 'vite'
import { releaseConfig } from './release-config.mjs'
const env = { ...loadEnv('production', process.cwd(), ''), ...process.env }
const config = releaseConfig(env)
const built = spawnSync('pnpm', ['exec', 'uni', 'build', '-p', 'mp-weixin'], { stdio: 'inherit', env: { ...env, CI: 'true', VITE_API_ORIGIN: config.origin, VITE_PROMOTION_ORIGIN: config.promotion } })
if (built.error) throw built.error
if (built.status !== 0) process.exit(built.status || 1)
const path = 'dist/build/mp-weixin/project.config.json'
const project = JSON.parse(readFileSync(path, 'utf8'))
project.appid = config.appid
project.setting.urlCheck = true
writeFileSync(path, JSON.stringify(project, null, 2) + '\n')
console.log('发布配置已校验并写入；仍需微信后台配置和真机验收。')
