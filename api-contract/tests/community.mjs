import assert from 'node:assert/strict'
import { mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import ts from 'typescript'
const source = await readFile(new URL('../src/modules/community/index.ts', import.meta.url), 'utf8')
const entry = ts.createSourceFile('index.ts', source, ts.ScriptTarget.ES2022, true)
assert.ok(entry.statements.length > 0 && entry.statements.every(statement => ts.isExportDeclaration(statement) && statement.moduleSpecifier), 'Community index must only re-export modules')
const output = await mkdtemp(join(tmpdir(), 'crowd-community-contract-'))
let community
let domains
try {
  const program = ts.createProgram([fileURLToPath(new URL('../src/modules/community/index.ts', import.meta.url))], {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.ES2022,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    lib: ['lib.es2022.d.ts'],
    types: [],
    strict: true,
    noEmitOnError: true,
    outDir: output,
    rootDir: fileURLToPath(new URL('../src/modules', import.meta.url)),
  })
  const result = program.emit()
  const diagnostics = [...ts.getPreEmitDiagnostics(program), ...result.diagnostics]
  assert.equal(diagnostics.length, 0, diagnostics.map(d => ts.flattenDiagnosticMessageText(d.messageText, '\n')).join('\n'))
  await writeFile(join(output, 'package.json'), JSON.stringify({ type: 'module' }))
  community = await import(pathToFileURL(join(output, 'community/index.js')).href)
  for (const domain of ['identity', 'catalog', 'attribution', 'tasks', 'governance', 'common']) {
    const domainSource = await readFile(new URL(`../src/modules/${domain}/index.ts`, import.meta.url), 'utf8')
    const domainEntry = ts.createSourceFile('index.ts', domainSource, ts.ScriptTarget.ES2022, true)
    assert.ok(domainEntry.statements.every(statement => ts.isExportDeclaration(statement) && statement.moduleSpecifier), `${domain} index must only re-export modules`)
    const directory = new URL(`../src/modules/${domain}/`, import.meta.url)
    for (const filename of await readdir(directory)) {
      if (!filename.endsWith('.ts')) continue
      const file = ts.createSourceFile(filename, await readFile(new URL(filename, directory), 'utf8'), ts.ScriptTarget.ES2022, true)
      for (const statement of file.statements) {
        if (!(ts.isImportDeclaration(statement) || ts.isExportDeclaration(statement)) || !statement.moduleSpecifier) continue
        const path = statement.moduleSpecifier.text
        assert.ok(!path.includes('/community/'), `${domain}/${filename} must not depend on the compatibility domain`)
        if (domain === 'common') assert.ok(path.startsWith('./'), `common/${filename} must not depend on business domains`)
      }
    }
  }
  domains = Object.fromEntries(await Promise.all(['identity', 'catalog', 'attribution', 'tasks', 'governance'].map(async domain => [domain, await import(pathToFileURL(join(output, domain, 'index.js')).href)])))
} finally {
  await rm(output, { recursive: true, force: true })
}
assert.deepEqual(Object.keys(community).sort(), ['BusinessError', 'createCommunity', 'taskStatusNames'])
const { createCommunity, BusinessError } = community
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

const tasks = { items: [{ id: 'claim', version: 2, status: 'in_progress', template: {}, reserved_cents: 101 }], next_cursor: null, server_time: 1800000000 }
const taskApi = createCommunity(async r => { request = r; return { data: tasks, request_id: 'tasks' } })
const filter = JSON.stringify({ status: 'submitted', cursor: 'a/b?x=1' })
assert.deepEqual(await taskApi.read('my_tasks', filter), tasks)
assert.equal(request.path, `/api/v1/community/read/my_tasks?target=${encodeURIComponent(filter)}`)
for (const bad of [{}, { items: [], next_cursor: 1 }, { items: [{ id: 'a', version: 1, status: 'claimed', template: {}, reserved_cents: 1.1 }], next_cursor: null }]) await assert.rejects(createCommunity(async () => ({ data: bad, request_id: 'bad-tasks' })).read('my_tasks'))
await taskApi.command('evidence_submit', { id: 'claim', version: 2, preview_id: 'preview', confirmed: true }, 'submission-key')
assert.equal(request.key, 'submission-key')
assert.equal(request.path, '/api/v1/community/commands/evidence_submit')
console.log('Task contract: integer digits, paginated responses, encoded filters and submission idempotency passed')
const taskRulesFixture = JSON.parse(await readFile(new URL('../fixtures/task-rules.json', import.meta.url), 'utf8'))
assert.deepEqual(await createCommunity(async () => ({ data: taskRulesFixture, request_id: 'rust-task-rules' })).read('task_rules'), taskRulesFixture)
const invalidPriceRule = structuredClone(taskRulesFixture)
invalidPriceRule.prices[0].content.amount_cents = 1.01
await assert.rejects(createCommunity(async () => ({ data: invalidPriceRule, request_id: 'unsafe-price' })).read('task_rules'))
console.log('Task rule contract: shared Rust serialization fixture and pricing guards passed')

const identityClient = domains.identity.createIdentity(async r => { request = r; return { data: session, request_id: 'domain-login' } })
assert.deepEqual(await identityClient.adminLogin({ username: 'admin', password: 'test-password' }), session)
assert.equal(request.path, '/api/v1/community/admin/login')
const domainCatalog = domains.catalog.createCatalog(async r => { request = r; return { data: publicFixture, request_id: 'domain-catalog' } })
assert.deepEqual(await domainCatalog.catalog({ min_cents: 0 }), publicFixture)
assert.equal(request.path, '/api/v1/catalog?min_cents=0')
await assert.rejects(domains.catalog.createCatalog(async () => ({ data: invalidAmount, request_id: 'bad-domain-catalog' })).catalog())
const attributionClient = domains.attribution.createAttribution(async r => { request = r; return { data: {}, request_id: 'domain-attribution' } })
await attributionClient.touchBind({ touch_token: 'visit' }, 'bind-key')
assert.equal(request.path, '/api/v1/community/commands/touch_bind')
assert.equal(request.key, 'bind-key')
const tasksClient = domains.tasks.createTasks(async r => { request = r; return { data: tasks, request_id: 'domain-tasks' } })
assert.deepEqual(await tasksClient.queue(filter), tasks)
assert.equal(request.path, `/api/v1/community/read/task_queue?target=${encodeURIComponent(filter)}`)
await tasksClient.evidenceSubmit({ id: 'claim', version: 2 }, 'domain-evidence-key')
assert.equal(request.path, '/api/v1/community/commands/evidence_submit')
assert.equal(request.key, 'domain-evidence-key')
await assert.rejects(domains.tasks.createTasks(async () => ({ data: invalidPriceRule, request_id: 'bad-domain-price' })).rules())
console.log('Independent domain clients: login, catalog guards, attribution binding, task queries and command idempotency passed')
assert.equal(domainCatalog.touch, undefined)
assert.equal(tasksClient.login, undefined)
assert.equal(identityClient.catalog, undefined)
const governanceClient = domains.governance.createGovernance(async r => { request = r; return { data: [], request_id: 'domain-audit' } })
assert.deepEqual(await governanceClient.audit('owner/a?scope=*'), [])
assert.equal(request.path, '/api/v1/community/read/audit?target=owner%2Fa%3Fscope%3D*')
await assert.rejects(domains.identity.createIdentity(async () => ({ error: { code: 'FORBIDDEN', message: '无权访问', request_id: 'domain-trace' } })).me(), e => e instanceof BusinessError && e.detail.request_id === 'domain-trace')
console.log('Domain boundaries: exports-only entries, dependency direction, isolated APIs and shared error identity passed')
