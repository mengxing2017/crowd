import { test, expect, type Page } from '@playwright/test'
async function fixture(page: Page, roles: string[]) {
 let application: Record<string, unknown> = { id: 'application-1', owner: 'alice', version: 2, status: 'ready', amount_cents: 600, currency: 'CNY', rule: { id: 'rule', version: 1, registration_verification: true }, receipt_snapshot: { id: 'receipt-old', owner: 'alice', version: 2, method: 'wechat', official_verified: false }, created_at: 100, updated_at: 100, handler: null, registration: null, events: [] }
 const commands: { action: string; body: Record<string, unknown>; key?: string }[] = []
 let failRegistration = true
 await page.route('**/api/v1/community/**', async route => {
  const request = route.request(), path = new URL(request.url()).pathname
  const json = (data: unknown) => route.fulfill({ json: { data, request_id: 'finance-browser' } })
  if (path.endsWith('/admin/login')) return json({ token: 'a'.repeat(48), user_id: 'staff', expires_in: 7200 })
  if (path.endsWith('/read/me')) return json({ id: 'staff', nickname: '测试管理员', avatar: null, status: 'active', roles, scope: ['*'], version: 1, receipts: [], restrictions: [] })
  if (path.endsWith('/read/rules') || path.endsWith('/read/withdrawal_rules')) return json([])
  if (path.endsWith('/read/withdrawal_queue')) return json([application])
  if (path.endsWith('/read/commission_summary')) return json({ owner: 'alice', currency: 'CNY', version: 1, pending_cents: 0, confirmed_cents: 1000, available_cents: 400, held_cents: 600, disputed_cents: 0, settled_cents: 0, consistent: true, notice: '非存款余额' })
  if (path.endsWith('/read/commission_entries')) return json({ items: [], next_cursor: null, total: 0 })
  if (path.endsWith('/read/commission_proposals') || path.endsWith('/read/commission_disputes')) return json([])
  if (path.endsWith('/files/settlement_voucher')) return json({ id: 'private-voucher', purpose: 'settlement_voucher', official_verified: false })
  if (path.includes('/commands/')) {
   const action = path.split('/').at(-1)!, body = request.postDataJSON() as Record<string, unknown>
   commands.push({ action, body, key: request.headers()['idempotency-key'] })
   if (action === 'settlement_claim') application = { ...application, status: 'processing', handler: 'staff', version: 3 }
   if (action === 'settlement_unknown') application = { ...application, status: 'unknown', version: 4 }
   if (action === 'settlement_register') { application = { ...application, status: 'awaiting_verification', version: 5, registration: { id: 'registration', amount_cents: 600, paid_at: 1000, recorded_at: 1100, official_verified: false } }; if (failRegistration) { failRegistration = false; return route.abort('failed') } }
   if (action === 'withdrawal_receipt_reveal') return json({ id: application.id, receipt_snapshot: { ...application.receipt_snapshot as object, file_id: 'private-original' }, registration: null, official_verified: false })
   return json(application)
  }
  throw new Error(`Unexpected ${path}`)
 })
 return commands
}
async function login(page: Page) { await page.goto('/'); await page.getByLabel('密码', { exact: true }).fill('browser-test-password'); await page.getByRole('button', { name: '登录', exact: true }).click(); await expect(page.getByText('欢迎回来，测试管理员')).toBeVisible() }
test('manual handler records uncertainty and retries original registration without a second payment request', async ({ page }) => {
 const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
 const commands = await fixture(page, ['manual_handler']); await login(page)
 await page.getByRole('menuitem', { name: '提现审核与人工结算' }).click()
 await page.getByRole('button', { name: '独占领取经办', exact: true }).click()
 await page.getByLabel('处理原因与核对说明').fill('独占领取，尚未线下付款')
 await page.getByRole('button', { name: '确认提交', exact: true }).click()
 await expect(page.getByText('人工处理中', { exact: true })).toBeVisible()
 await page.getByRole('button', { name: '登记结果待核实', exact: true }).click()
 await page.getByLabel('处理原因与核对说明').fill('线下结果尚不明确，保留原占用')
 await page.getByRole('button', { name: '确认提交', exact: true }).click()
 await expect(page.getByText('待核实', { exact: true })).toBeVisible()
 await page.getByRole('button', { name: '登记线下付款', exact: true }).click()
 await page.getByLabel('实际付款参考金额（整数分）').fill('600')
 await page.getByLabel('线下付款时间（Unix 秒）').fill('1000')
 await page.locator('input[type=file]').setInputFiles({ name: 'voucher.png', mimeType: 'image/png', buffer: Buffer.from('test-image-fixture') })
 await expect(page.getByLabel('私有人工凭证')).toHaveValue('private-voucher')
 await page.getByLabel('已核对该申请原收款快照，未改付新资料').check()
 await page.getByLabel('凭证与金额相符（不符将转待核实）').check()
 await page.getByLabel('处理原因与核对说明').fill('核查原付款结果后登记，非官方验证')
 await page.getByRole('button', { name: '确认提交', exact: true }).click()
 await expect(page.getByText('上次提交结果尚未确认')).toBeVisible()
 await page.getByRole('button', { name: '关闭', exact: true }).click()
 await page.getByRole('button', { name: '原请求重试', exact: true }).click()
 await expect.poll(() => commands.filter(c => c.action === 'settlement_register').length).toBe(2)
 const registered = commands.filter(c => c.action === 'settlement_register')
 expect(registered[0]).toEqual(registered[1]); expect(registered[0].body).toMatchObject({ receipt_id: 'receipt-old', amount_cents: 600, version: 4 })
 expect(commands.every(c => !c.action.includes('transfer'))).toBe(true); expect(errors).toEqual([])
})
test('auditor sees numbers and queues but has no business mutation controls', async ({ page }) => {
 await fixture(page, ['auditor']); await login(page)
 await page.getByRole('menuitem', { name: '佣金数字与流水' }).click()
 await page.getByLabel('佣金用户编号').fill('alice'); await page.getByRole('button', { name: '查询数字及流水' }).click()
 await expect(page.getByText('流水与申请占用一致')).toBeVisible()
 await expect(page.getByRole('button', { name: '提交调整复核' })).toHaveCount(0)
 await page.getByRole('menuitem', { name: '提现审核与人工结算' }).click()
 await expect(page.getByRole('button', { name: '独占领取经办' })).toHaveCount(0)
 await expect(page.getByRole('button', { name: '预览批量审核' })).toHaveCount(0)
 await expect(page.getByRole('menuitem', { name: '提现申请规则' })).toHaveCount(0)
})
test('withdrawal rule editing requires explicit thresholds and independent publication', async ({ page }) => {
 const commands = await fixture(page, ['rule_manager']); await login(page)
 await page.getByRole('menuitem', { name: '提现申请规则' }).click()
 await expect(page.getByLabel('最低申请（分）')).toHaveValue('')
 for (const [label, value] of [['最低申请（分）', '100'], ['最高申请（分）', '1000'], ['每日次数', '3'], ['每日累计申请（分）', '3000'], ['累计申请上限（分）', '10000']]) await page.getByLabel(label, { exact: true }).fill(value)
 await page.getByLabel('规则依据', { exact: true }).fill('隔离浏览器测试规则')
 await page.getByRole('button', { name: '保存并提交独立复核' }).click()
 await expect.poll(() => commands.length).toBe(1)
 expect(commands[0]).toMatchObject({ action: 'withdrawal_rule_propose', body: { min_cents: 100, max_cents: 1000, registration_verification: true, separate_roles: true, second_review_cents: 0 } })
 expect(commands[0].key).toBeTruthy()
})
