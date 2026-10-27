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

const session = { token: 'a'.repeat(48), user_id: 'staff', expires_in: 7200 }
const staffApi = createCommunity(async r => { request = r; return { data: session, request_id: 'admin-login' } })
assert.deepEqual(await staffApi.adminLogin({ username: 'admin', password: 'test-password' }), session)
assert.equal(request.path, '/api/v1/community/admin/login')
assert.equal(request.method, 'POST')
assert.deepEqual(request.body, { username: 'admin', password: 'test-password' })
await assert.rejects(createCommunity(async () => ({ data: {}, request_id: 'bad-staff' })).adminLogin({ username: 'admin', password: 'wrong' }))

const catalog = { categories: [], products: [], revision: 1 }
const catalogApi = createCommunity(async r => { request = r; return { data: catalog, request_id: 'catalog' } })
assert.deepEqual(await catalogApi.catalog({ keyword: '产品 & A', min_cents: 100, task_status: 'available' }), catalog)
assert.equal(request.path, '/api/v1/catalog?keyword=%E4%BA%A7%E5%93%81%20%26%20A&min_cents=100&task_status=available')
await catalogApi.touch({ code: 'code', scene: 'miniapp' }, 'visit-key')
assert.equal(request.path, '/api/v1/catalog/touches')
assert.equal(request.key, 'visit-key')
for (const bad of [{}, { products: [], categories: [], revision: 1.5 }, { categories: [], products: [{ id: 'p', content: {} }], revision: 1 }]) {
  await assert.rejects(createCommunity(async () => ({ data: bad, request_id: 'bad-catalog' })).catalog())
}
console.log('Catalog contract: query encoding, visit idempotency and public response guards passed')
const publicFixture = JSON.parse(await readFile(new URL('../fixtures/catalog.json', import.meta.url), 'utf8'))
assert.deepEqual(await createCommunity(async () => ({ data: publicFixture, request_id: 'rust-catalog-fixture' })).catalog(), publicFixture)
const invalidAmount = structuredClone(publicFixture)
invalidAmount.products[0].content.tasks[0].commission_cents = 1.1
await assert.rejects(createCommunity(async () => ({ data: invalidAmount, request_id: 'invalid-cents' })).catalog())
