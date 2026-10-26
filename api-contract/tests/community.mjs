import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import ts from 'typescript'
const source = await readFile(new URL('../src/modules/community/index.ts', import.meta.url), 'utf8')
const compiled = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 } }).outputText
const { createCommunity, BusinessError } = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString('base64')}`)
const fixture = JSON.parse(await readFile(new URL('../fixtures/community-user.json', import.meta.url), 'utf8'))
let request
const api = createCommunity(async r => { request = r; return { data: fixture, request_id: 'fixture' } })
assert.deepEqual(await api.read('me'), fixture)
assert.equal(request.path, '/api/v1/community/read/me')
await api.read('audit', 'owner/a?scope=*')
assert.equal(request.path, '/api/v1/community/read/audit?target=owner%2Fa%3Fscope%3D*')
await api.command('profile', { nickname: 'A', version: 1 }, 'original-key')
assert.equal(request.key, 'original-key')
assert.equal(request.method, 'POST')
for (const bad of [null, {}, { data: {} }, { data: {}, request_id: 'bad-user' }]) {
  await assert.rejects(createCommunity(async () => bad).read('me'))
}
await assert.rejects(createCommunity(async () => ({ data: { token: 'forged', user_id: 'x', expires_in: 10 }, request_id: 'bad-login' })).redeem('ticket'))
await assert.rejects(createCommunity(async () => ({ error: { code: 'FORBIDDEN', message: '无权访问', request_id: 'trace' } })).read('audit'), e => e instanceof BusinessError && e.detail.request_id === 'trace')
console.log('Community contract: Rust fixture, envelope guards, request encoding, idempotency and error handling passed')
