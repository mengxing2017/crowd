import test from 'node:test'
import assert from 'node:assert/strict'
import { resolve } from 'node:path'
import { harness, root } from './harness.mjs'
import { releaseConfig } from '../scripts/release-config.mjs'
const deferred = () => { let resolve; let reject; const promise = new Promise((a, b) => { resolve = a; reject = b }); return { promise, resolve, reject } }
const tick = () => new Promise(resolve => setImmediate(resolve))

test('ordinary QR decodes exactly once and rejects untrusted domains/paths', () => {
  const { promotionSource } = harness().load('features/promotion/entry.ts')
  const origin = 'https://go.example.test'
  assert.equal(promotionSource({ q: encodeURIComponent(`${origin}/?source=code-1`) }, origin), 'code-1')
  assert.equal(promotionSource({ source: 'direct' }, ''), 'direct')
  for (const link of ['https://go.example.test.evil/?source=evil', 'https://go.example.test/other/?source=evil', 'http://go.example.test/?source=evil']) assert.throws(() => promotionSource({ q: encodeURIComponent(link) }, origin))
  assert.throws(() => promotionSource({ q: '%E0%A4%A' }, origin))
})

test('shared queue limits running work and releases slots after failure', async () => {
  const { createRequestQueue } = harness().load('shared/request-queue.ts')
  const schedule = createRequestQueue(2), gates = [deferred(), deferred(), deferred()]
  let count = 0, peak = 0
  const jobs = gates.map((gate, i) => schedule(async () => { peak = Math.max(peak, ++count); try { await gate.promise; if (i === 0) throw new Error('offline'); return i } finally { count-- } }).catch(() => 'failed'))
  assert.equal(count, 2); gates[0].resolve(); await tick(); assert.equal(count, 2)
  gates[1].resolve(); gates[2].resolve(); assert.deepEqual(await Promise.all(jobs), ['failed', 1, 2]); assert.equal(peak, 2)
})

test('401 clears every view once; an old response cannot clear a newer login', async () => {
  const calls = []
  const h = harness({}, { request: options => calls.push(options) })
  const session = h.load('features/session/api/index.ts')
  let expired = 0; session.onSessionExpired(() => expired++)
  session.setToken('old')
  const old = session.request({ url: 'https://api.example.test/old' })
  session.setToken('new')
  calls.shift().success({ statusCode: 401, data: {} })
  await assert.rejects(old, /会话已变化/); assert.equal(expired, 0)
  const current = session.request({ url: 'https://api.example.test/new' })
  calls.shift().success({ statusCode: 401, data: {} })
  await assert.rejects(current, /登录已过期/); assert.equal(expired, 1); assert.equal(session.authHeader().Authorization, 'Bearer ')
})

test('private downloads arriving after hide or close are removed, including after show', async () => {
  const files = [deferred(), deferred(), deferred()]
  let index = 0
  const removed = []
  const h = harness({ [resolve(root, 'features/files/api/index.ts')]: { download: () => files[index++].promise, removeTempFile: path => removed.push(path) } })
  let value = 'visible'
  const scope = h.load('features/files/private-files.ts').usePrivateFiles(() => { value = '' })
  const late = scope.download('private')
  h.hooks.hide.forEach(f => f()); h.hooks.show.forEach(f => f()); files[0].resolve('late-path')
  assert.equal(await late, undefined); assert.equal(value, ''); assert.deepEqual(removed, ['late-path'])
  const shown = scope.download('private'); files[1].resolve('current-path'); assert.equal(await shown, 'current-path')
  scope.clear(); assert.ok(removed.includes('current-path'))
  const closed = scope.download('private'); scope.clear(); files[2].resolve('closed-path'); assert.equal(await closed, undefined); assert.ok(removed.includes('closed-path'))
})

test('restart recovery stores only owner/action/key, never payload or credentials', () => {
  const h = harness(), recovery = h.load('features/platform/recovery.ts')
  recovery.rememberCommand({ owner: 'alice', action: 'receipt', key: 'same-key', body: { account: 'private', token: 'secret' } })
  const serialized = JSON.stringify([...h.storage.values()])
  assert.equal(serialized.includes('private'), false); assert.equal(serialized.includes('secret'), false)
  assert.deepEqual(JSON.parse(JSON.stringify(recovery.restoreCommand('alice'))), { owner: 'alice', action: 'receipt', key: 'same-key' })
  assert.equal(recovery.restoreCommand('bob'), null)
  recovery.forgetCommand({ owner: 'alice', key: 'older-key' }); assert.ok(recovery.restoreCommand('alice'))
  recovery.forgetCommand({ owner: 'alice', key: 'same-key' }); assert.equal(recovery.restoreCommand('alice'), null)
})

function pageHarness({ command, me } = {}) {
  const path = resolve(root, 'pages/index/index.vue')
  class BusinessError extends Error {}
  const overrides = { expose: { [path]: ['submit', 'recoverCommand', 'pending', 'me', 'message', 'refreshWarning', 'fullAccount', 'reveal', 'hidePrivate'] },
    [resolve(root, 'features/platform/api/index.ts')]: { BusinessError, executeCommand: command || (async () => ({ id: 'committed' })), platform: { commandResult: async () => ({ id: 'restored', account: 'private-account' }) } },
    [resolve(root, 'features/identity/api/index.ts')]: { identity: { me: me || (async () => { throw new Error('refresh offline') }) } },
  }
  const h = harness(overrides)
  return { ...h, page: h.load('pages/index/index.vue') }
}

test('committed command result survives a subsequent refresh outage', async () => {
  const { page } = pageHarness()
  page.me.value = { id: 'alice' }
  assert.equal((await page.submit('withdrawal_preview', {})).id, 'committed')
  assert.equal(page.pending.value, null); assert.match(page.refreshWarning.value, /提交已成功/)
})

test('uncertain submission retains original key/body; querying original dispatches recovered value', async () => {
  const { page, load } = pageHarness({ command: async () => { throw new Error('response lost') } })
  page.me.value = { id: 'alice' }
  await assert.rejects(page.submit('withdrawal_preview', { amount_cents: 100 }), /response lost/)
  const key = page.pending.value.key
  let recovered
  load('features/platform/recovery.ts').onCommandRecovered(value => { recovered = value })
  await page.recoverCommand()
  assert.equal(recovered.action, 'withdrawal_preview'); assert.equal(recovered.result.id, 'restored'); assert.equal(page.pending.value, null)
  assert.ok(key)
})

test('receipt reveal response after hide never restores sensitive text', async () => {
  const reply = deferred()
  const { page, hooks } = pageHarness({ command: () => reply.promise })
  page.me.value = { id: 'alice' }
  const work = page.reveal({ id: 'receipt' })
  hooks.hide.forEach(f => f()); reply.resolve({ account: 'secret-account' })
  await work; assert.equal(page.fullAccount.value, '')
})

test('release preflight rejects tourist/HTTP/IP config and accepts explicit HTTPS domains', () => {
  const good = { VITE_MP_WEIXIN_APPID: 'wx0123456789abcdef', VITE_API_ORIGIN: 'https://api.example.test', VITE_PROMOTION_ORIGIN: 'https://go.example.test' }
  assert.equal(releaseConfig(good).appid, good.VITE_MP_WEIXIN_APPID)
  for (const bad of [{ VITE_MP_WEIXIN_APPID: '' }, { VITE_API_ORIGIN: 'http://127.0.0.1:8080' }, { VITE_API_ORIGIN: 'https://127.0.0.1' }, { VITE_API_ORIGIN: 'https://user:pass@api.example.test' }, { VITE_PROMOTION_ORIGIN: '' }]) assert.throws(() => releaseConfig({ ...good, ...bad }))
})

test('supplement image fields reject additions/removals outside requested fields', async () => {
  const path = resolve(root, 'features/tasks/TaskCenter.vue')
  const h = harness({ expose: { [path]: ['selected', 'draft', 'choose', 'removeImage'] }, props: { run: async () => undefined } })
  const task = h.load('features/tasks/TaskCenter.vue')
  task.selected.value = { status: 'supplement', supplement_fields: ['requested'] }
  task.draft.value = { fields: { locked: ['original'], requested: ['replaceable'] } }
  await assert.rejects(task.choose('locked'), /不在本次补交范围/)
  task.removeImage('locked', 'original'); assert.equal(task.draft.value.fields.locked.length, 1)
  task.removeImage('requested', 'replaceable'); assert.equal(task.draft.value.fields.requested.length, 0)
})

test('support selector follows commission and rejected-task cursors', async () => {
  const path = resolve(root, 'features/support/SupportCenter.vue'), calls = []
  const h = harness({ expose: { [path]: ['loadChoices', 'choices', 'choiceCursor', 'category'] }, props: { owner: 'alice', run: async () => undefined },
    [resolve(root, 'features/commissions/api/index.ts')]: { commissions: { entries: async filter => { calls.push(filter); return filter === 'alice|0' ? { items: [{ id: 'first', amount_cents: 1 }], next_cursor: 7 } : { items: [{ id: 'older', amount_cents: 2 }], next_cursor: null } } } },
    [resolve(root, 'features/tasks/api/index.ts')]: { tasks: { mine: async filter => { const query = JSON.parse(filter); assert.equal(query.status, 'rejected'); return { items: [{ id: 'rejected-old', status: 'rejected' }], next_cursor: null } } } },
  })
  const support = h.load('features/support/SupportCenter.vue')
  support.category.value = 'commission'; await support.loadChoices(); await support.loadChoices(true)
  assert.deepEqual(calls, ['alice|0', 'alice|7']); assert.equal(support.choices.value[1].id, 'older'); assert.equal(support.choiceCursor.value, null)
  support.category.value = 'appeal'; await support.loadChoices(); assert.equal(support.choices.value[0].id, 'rejected-old')
})

test('risk request failure preserves unknown/error state instead of successful empty', async () => {
  const path = resolve(root, 'features/risk/RiskCenter.vue')
  const h = harness({ expose: { [path]: ['load', 'loading', 'loaded', 'error'] }, props: { run: async () => undefined },
    [resolve(root, 'features/risk/api/index.ts')]: { risk: { myRisk: async () => [] } },
    [resolve(root, 'features/commissions/api/index.ts')]: { commissions: { adjustments: async () => { throw new Error('offline') } } },
  })
  const risk = h.load('features/risk/RiskCenter.vue')
  await assert.rejects(risk.load(), /offline/); assert.equal(risk.loaded.value, false); assert.equal(risk.loading.value, false); assert.equal(risk.error.value, 'offline')
})

test('newer catalog search wins over a slow previous response', async () => {
  const path = resolve(root, 'pages/index/index.vue'), first = deferred(), second = deferred()
  let calls = 0
  const h = harness({ expose: { [path]: ['loadCatalog', 'catalog'] },
    [resolve(root, 'features/catalog/api/index.ts')]: { catalog: { catalog: () => (++calls === 1 ? first.promise : second.promise) } },
  })
  const page = h.load('pages/index/index.vue')
  const a = page.loadCatalog(), b = page.loadCatalog()
  second.resolve({ products: [{ id: 'new' }] }); await b
  first.resolve({ products: [{ id: 'old' }] }); await a
  assert.equal(page.catalog.value.products[0].id, 'new')
})

test('retry uses the original key and frozen body even when the form changes', async () => {
  const calls = []
  const { page } = pageHarness({ command: async (action, body, key) => { calls.push({ action, body: JSON.parse(JSON.stringify(body)), key }); if (calls.length === 1) throw new Error('response lost'); return { id: 'once' } } })
  page.me.value = { id: 'alice' }
  const body = { evidence: { text: 'original' } }
  await assert.rejects(page.submit('evidence_save', body))
  body.evidence.text = 'edited'
  await page.recoverCommand(true)
  assert.equal(calls.length, 2); assert.deepEqual(calls[0], calls[1]); assert.equal(calls[1].body.evidence.text, 'original')
})

test('a late privacy copy cannot repopulate a closed preview', async () => {
  const path = resolve(root, 'features/privacy/PrivacyCenter.vue'), reply = deferred()
  const h = harness({ expose: { [path]: ['viewCopy', 'clear', 'content'] }, props: { run: async () => undefined },
    [resolve(root, 'features/privacy/api/index.ts')]: { privacy: { privacyCopy: () => reply.promise } },
  })
  const view = h.load('features/privacy/PrivacyCenter.vue'), work = view.viewCopy('copy')
  view.clear(); reply.resolve({ content: { account: 'secret' } }); await work; assert.equal(view.content.value, '')
})

test('restored withdrawal preview populates its own component and requires fresh confirmation', () => {
  const path = resolve(root, 'features/withdrawals/WithdrawalCenter.vue')
  const h = harness({ expose: { [path]: ['preview', 'confirmed', 'receiptId', 'amount'] }, props: { me: { id: 'alice', receipts: [] }, busy: false, blocked: false } }, { request: () => undefined })
  const view = h.load('features/withdrawals/WithdrawalCenter.vue')
  view.confirmed.value = true
  h.load('features/platform/recovery.ts').publishRecovered({ action: 'withdrawal_preview', result: { id: 'original-preview', receipt_id: 'receipt-v1', amount_cents: 1234 } })
  assert.equal(view.preview.value.id, 'original-preview'); assert.equal(view.receiptId.value, 'receipt-v1'); assert.equal(view.amount.value, '12.34'); assert.equal(view.confirmed.value, false)
})

test('legal long task text reaches the saved draft; limits use Unicode code points', async () => {
  const path = resolve(root, 'features/tasks/TaskCenter.vue'), saved = []
  const h = harness({ expose: { [path]: ['selected', 'draft', 'preview'] }, props: { run: async () => undefined, submit: async (action, body) => { saved.push({ action, body }); return {} } },
    [resolve(root, 'features/tasks/api/index.ts')]: { taskStatusNames: {}, tasks: { mine: async () => ({ items: [] }), detail: async () => ({ id: 'task', version: 1, template: { fields: [] } }) } },
  })
  const view = h.load('features/tasks/TaskCenter.vue')
  view.selected.value = { id: 'task', version: 1, template: { fields: [{ id: 'text', kind: 'text', required: true, label: '说明', max_length: 500 }] } }
  view.draft.value = { fields: { text: '😀'.repeat(200) } }
  await view.preview(); assert.equal([...saved[0].body.evidence.fields.text].length, 200)
  view.selected.value = { id: 'task', version: 1, template: { fields: [{ id: 'text', kind: 'text', required: true, label: '说明', max_length: 10 }] } }
  await assert.rejects(view.preview(), /超过最大字数/)
})

test('oversized image is rejected before reading its entire contents or sending requests', async () => {
  let reads = 0, requests = 0
  const h = harness({}, { getFileSystemManager: () => ({ getFileInfo: options => options.success({ size: 6 * 1024 * 1024 }), readFile: () => { reads++ } }), request: () => { requests++ } })
  await assert.rejects(h.load('features/files/api/index.ts').upload('large-image', 'task_evidence:t'), /5 MiB/)
  assert.equal(reads, 0); assert.equal(requests, 0)
})

test('image picking prefers chooseMedia and preserves an older-library fallback and cancel feedback', async () => {
  let preferred = 0, legacy = 0
  const modern = harness({}, { canIUse: () => true, chooseMedia: options => { preferred++; assert.equal(options.mediaType[0], 'image'); options.success({ tempFiles: [{ tempFilePath: 'modern-image' }] }) } })
  assert.equal(await modern.load('features/files/api/index.ts').chooseImagePath(), 'modern-image')
  const old = harness({}, { canIUse: () => false, chooseImage: options => { legacy++; options.success({ tempFilePaths: ['legacy-image'] }) } })
  assert.equal(await old.load('features/files/api/index.ts').chooseImagePath(true), 'legacy-image')
  assert.equal(preferred, 1); assert.equal(legacy, 1)
  const cancel = harness({}, { canIUse: () => true, chooseMedia: options => options.fail({ errMsg: 'chooseMedia:fail cancel' }) })
  await assert.rejects(cancel.load('features/files/api/index.ts').chooseImagePath(), /已取消选择/)
})
