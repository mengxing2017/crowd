import { expect, test, type Page } from '@playwright/test'
interface FixtureRule { id: string; kind: string; status: string; version: number; content: Record<string, unknown> }
async function fixtures(page: Page, roles = ['rule_manager', 'identity_reviewer', 'promotion_manager', 'auditor']) {
  const rules: FixtureRule[] = [{ id: 'agreement-1', kind: 'agreements', status: 'published', version: 1, content: { service: '测试协议正文', privacy: '测试隐私正文' } }]
  const commands: { action: string; body: Record<string, unknown>; key: string | undefined }[] = []
  const catalog: { categories: Record<string, unknown>[]; products: Record<string, unknown>[]; revision: number; server_time: number } = { categories: [], products: [], revision: 0, server_time: 1800000000 }
  let uncertain = false
  await page.route('**/api/v1/community/**', async route => {
    const request = route.request()
    const path = new URL(request.url()).pathname
    const json = (data: unknown) => route.fulfill({ json: { data, request_id: 'browser-fixture' } })
    if (path.endsWith('/admin/login')) return json({ token: 'a'.repeat(48), user_id: 'staff', expires_in: 7200 })
    expect(request.headers().authorization).toBe('Bearer ' + 'a'.repeat(48))
    if (path.endsWith('/read/me')) return json({ id: 'staff', nickname: '测试管理员', avatar: null, status: 'active', roles, scope: ['*'], version: 1, receipts: [], restrictions: [] })
    if (path.endsWith('/read/rules')) return json(rules)
    if (path.endsWith('/read/managed_codes')) return json([{ id: 'test-code', code: { owner: 'target', active: true, version: 1, product: null, task: null } }])
    if (path.endsWith('/read/users')) return json({ id: 'target', status: 'active', roles: ['user'], scope: ['target'], version: 1, restrictions: [] })
    if (path.endsWith('/read/eligibility')) return json({ version: 0 })
    if (path.endsWith('/read/catalog_admin')) return json(catalog)
    if (path.endsWith('/read/attribution_admin') || path.endsWith('/read/attribution_policies')) return json({ policies: [], touch_count: 0, disputes: [] })
    if (path.endsWith('/read/audit')) return json([])
    if (path.includes('/commands/')) {
      const body = request.postDataJSON() as Record<string, unknown>
      const action = path.split('/').at(-1)!
      commands.push({ action, body, key: request.headers()['idempotency-key'] })
      if (action === 'category_save') catalog.categories.push({ ...body, id: 'category-1', version: 1 })
      if (action === 'product_save') catalog.products.push({ id: 'product-1', version: 1, draft: body.content, releases: [], archived: false, offline: true })
      if (action === 'rule_save') {
        rules.push({ id: 'draft-2', kind: body.kind as string, version: 2, status: 'draft', content: body.content as Record<string, unknown> })
      }
      if (action === 'rule_publish') { rules.forEach(rule => { rule.status = rule.id === body.id ? 'published' : 'disabled' }) }
      if (action === 'code_rotate' && !uncertain) { uncertain = true; return route.abort('failed') }
      return json({ id: body.id || 'saved' })
    }
    throw new Error(`Unexpected fixture request ${path}`)
  })
  return commands
}
async function login(page: Page) {
  await page.goto('/')
  await page.getByLabel('密码', { exact: true }).fill('browser-test-password')
  await page.getByRole('button', { name: '登录', exact: true }).click()
  await expect(page.getByText('欢迎回来，测试管理员')).toBeVisible()
}

test('login, rule draft and publish use real UI forms and current versions', async ({ page }, testInfo) => {
  const errors: string[] = []; page.on('pageerror', error => errors.push(error.message))
  const commands = await fixtures(page)
  await page.goto('/')
  await page.screenshot({ path: testInfo.outputPath('login-desktop.png'), fullPage: true })
  await login(page)
  await page.screenshot({ path: testInfo.outputPath('workspace-desktop.png'), fullPage: true })
  await page.getByRole('menuitem', { name: '业务规则' }).click()
  await page.getByRole('button', { name: '新建规则草稿' }).click()
  await page.getByLabel('服务协议', { exact: true }).fill('更新后的测试服务协议')
  await page.getByLabel('隐私说明', { exact: true }).fill('更新后的测试隐私说明')
  await page.getByRole('button', { name: '保存草稿', exact: true }).click()
  await expect(page.getByRole('row').filter({ hasText: 'V2' })).toBeVisible()
  await page.getByRole('row').filter({ hasText: 'V2' }).getByRole('button', { name: '发布', exact: true }).click()
  await page.getByRole('button', { name: '确 定' }).click()
  await expect(page.getByRole('row').filter({ hasText: 'V2' })).toContainText('已发布')
  expect(commands[0]).toMatchObject({ action: 'rule_save', body: { kind: 'agreements', version: 1, content: { service: '更新后的测试服务协议', privacy: '更新后的测试隐私说明' } } })
  expect(commands[0].key).toBeTruthy()
  expect(errors).toEqual([])
})

test('limited role menu and direct hashes do not expose rule editing', async ({ page }) => {
  await fixtures(page, ['auditor'])
  await login(page)
  await expect(page.getByRole('menuitem', { name: '业务规则' })).toHaveCount(0)
  await page.goto('/#/rules')
  await expect(page.getByRole('button', { name: '新建规则草稿' })).toHaveCount(0)
  await page.reload()
  // Refresh restores the session, while retaining the role boundary.
  await expect(page.getByText('欢迎回来，测试管理员')).toBeVisible()
  await expect(page.getByRole('button', { name: '新建规则草稿' })).toHaveCount(0)
  await page.getByRole('menuitem', { name: '审计日志' }).click()
  await page.getByLabel('业务对象', { exact: true }).fill('*')
  await page.getByRole('button', { name: '查询日志' }).click()
  await expect(page.getByText('暂无符合条件的审计记录')).toBeVisible()
})

test('uncertain mutation retries original body and key', async ({ page }) => {
  const commands = await fixtures(page)
  await login(page)
  await page.getByRole('menuitem', { name: '推广码管理' }).click()
  await page.getByLabel('所属用户', { exact: true }).fill('target')
  await page.getByRole('button', { name: '查询推广码' }).click()
  await page.getByRole('button', { name: '撤销并换发', exact: true }).click()
  await page.getByLabel('换发原因', { exact: true }).fill('测试换码')
  await page.getByLabel('审批依据', { exact: true }).fill('test-approval')
  await page.getByRole('button', { name: '确认换发', exact: true }).click()
  await expect(page.getByText('上次提交结果尚未确认')).toBeVisible()
  await page.getByRole('button', { name: /^取\s*消$/ }).click()
  await page.getByRole('button', { name: '原请求重试', exact: true }).click()
  await expect(page.getByText('上次提交结果尚未确认')).toHaveCount(0)
  expect(commands).toHaveLength(2)
  expect(commands[0]).toEqual(commands[1])
})

test('mobile login and workspace fit the viewport', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await fixtures(page)
  await page.goto('/')
  await page.screenshot({ path: testInfo.outputPath('login-mobile.png'), fullPage: true })
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
  await login(page)
  await page.screenshot({ path: testInfo.outputPath('workspace-mobile.png'), fullPage: true })
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
})

test('user permissions submit the queried version and explicit scope', async ({ page }) => {
  const commands = await fixtures(page)
  await login(page)
  await page.getByRole('menuitem', { name: '用户与权限' }).click()
  await page.getByLabel('用户编号', { exact: true }).fill('target')
  await page.getByRole('button', { name: /查询/ }).click()
  await expect(page.getByText('用户档案', { exact: true })).toBeVisible()
  await page.getByLabel('变更原因', { exact: true }).fill('测试岗位核对')
  await page.getByRole('button', { name: '保存岗位与范围' }).click()
  await expect.poll(() => commands.length).toBe(1)
  expect(commands[0]).toMatchObject({ action: 'access', body: { user_id: 'target', version: 1, roles: ['user'], scope: ['target'], reason: '测试岗位核对' } })
})

test('confirmed draft closes even if the subsequent refresh fails', async ({ page }) => {
  const commands = await fixtures(page)
  let reads = 0
  await page.route('**/api/v1/community/read/rules', async route => {
    if (++reads === 2) return route.fulfill({ status: 503, json: { error: { code: 'SERVICE_UNAVAILABLE', message: '规则刷新失败，请刷新页面', request_id: 'refresh-failure' } } })
    return route.fallback()
  })
  await login(page)
  await page.getByRole('menuitem', { name: '业务规则' }).click()
  await page.getByRole('button', { name: '新建规则草稿' }).click()
  await page.getByLabel('服务协议', { exact: true }).fill('测试服务协议')
  await page.getByLabel('隐私说明', { exact: true }).fill('测试隐私说明')
  await page.getByRole('button', { name: '保存草稿', exact: true }).click()
  await expect(page.getByText(/规则刷新失败，请刷新页面/)).toBeVisible()
  await expect(page.getByRole('button', { name: '保存草稿', exact: true })).toHaveCount(0)
  await expect(page.getByText('上次提交结果尚未确认')).toHaveCount(0)
  expect(commands).toHaveLength(1)
})


test('refresh keeps the authenticated route and logout clears persisted session', async ({ page }) => {
  await fixtures(page)
  await login(page)
  await page.getByRole('menuitem', { name: '业务规则' }).click()
  await page.reload()
  await expect(page.getByRole('button', { name: '新建规则草稿' })).toBeVisible()
  await expect(page.getByRole('button', { name: '登录', exact: true })).toHaveCount(0)
  expect(await page.evaluate(() => sessionStorage.getItem('crowd.web.session'))).toBe('a'.repeat(48))
  await page.locator('.account-menu').hover()
  await page.getByRole('menuitem', { name: '退出登录' }).click()
  await expect(page.getByRole('button', { name: '登录', exact: true })).toBeVisible()
  expect(await page.evaluate(() => sessionStorage.getItem('crowd.web.session'))).toBeNull()
  await page.reload()
  await expect(page.getByRole('button', { name: '登录', exact: true })).toBeVisible()
})

test('expired restored session is cleared and requires login', async ({ page }) => {
  await fixtures(page)
  await login(page)
  await page.route('**/api/v1/community/read/me', route => route.fulfill({ status: 401, json: { error: { code: 'SESSION_EXPIRED', message: '会话已失效，请重新登录', request_id: 'expired-session' } } }))
  await page.reload()
  await expect(page.getByRole('button', { name: '登录', exact: true })).toBeVisible()
  await expect(page.getByText(/会话已失效，请重新登录/)).toBeVisible()
  expect(await page.evaluate(() => sessionStorage.getItem('crowd.web.session'))).toBeNull()
})

test('transient restore failure allows retry without losing the session', async ({ page }) => {
  await fixtures(page)
  await login(page)
  await page.route('**/api/v1/community/read/me', route => route.fulfill({ status: 503, json: { error: { code: 'SERVICE_UNAVAILABLE', message: '会话服务暂不可用', request_id: 'restore-failure' } } }))
  await page.reload()
  await expect(page.getByText('暂时无法恢复登录状态')).toBeVisible()
  expect(await page.evaluate(() => sessionStorage.getItem('crowd.web.session'))).toBe('a'.repeat(48))
  await page.unroute('**/api/v1/community/read/me')
  await page.getByRole('button', { name: '重试恢复' }).click()
  await expect(page.getByText('欢迎回来，测试管理员')).toBeVisible()
})


test('product classification and draft forms use versioned commands', async ({ page }) => {
  const commands = await fixtures(page, ['operator', 'product_reviewer'])
  await login(page)
  await page.getByRole('menuitem', { name: '产品目录与发布' }).click()
  await page.getByRole('button', { name: '新增分类', exact: true }).click()
  await page.getByLabel('分类名称', { exact: true }).fill('测试产品分类')
  await page.getByLabel('变更原因', { exact: true }).fill('运营维护分类')
  await page.getByRole('button', { name: '保存分类', exact: true }).click()
  await expect(page.getByRole('row').filter({ hasText: '测试产品分类' })).toBeVisible()
  await page.getByRole('button', { name: '新增产品', exact: true }).click()
  await page.getByLabel('产品名称', { exact: true }).fill('待审核的产品草稿')
  await page.getByLabel('简介', { exact: true }).fill('测试简介')
  await page.getByRole('button', { name: '保存产品草稿', exact: true }).click()
  await expect(page.getByRole('row').filter({ hasText: '待审核的产品草稿' })).toBeVisible()
  expect(commands[0]).toMatchObject({ action: 'category_save', body: { version: 0, name: '测试产品分类', enabled: true } })
  expect(commands[1]).toMatchObject({ action: 'product_save', body: { version: 0, content: { name: '待审核的产品草稿', regions: [], qualifications: [] } } })
})

test('publishing reviewer can inspect products but cannot edit drafts', async ({ page }) => {
  await fixtures(page, ['product_reviewer'])
  await login(page)
  await page.getByRole('menuitem', { name: '产品目录与发布' }).click()
  await expect(page.getByText('产品与发布版本', { exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: '新增产品', exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: '新增分类', exact: true })).toHaveCount(0)
})

test('attribution rule manager explicitly chooses window and first or last touch', async ({ page }) => {
  const commands = await fixtures(page, ['rule_manager'])
  await login(page)
  await page.getByRole('menuitem', { name: '推广归因' }).click()
  await page.getByLabel('归因有效窗口（秒）', { exact: true }).fill('600')
  await page.getByLabel('多触点策略', { exact: true }).click()
  await page.getByText('末触', { exact: true }).click()
  await page.getByRole('button', { name: '发布新策略版本', exact: true }).click()
  await expect.poll(() => commands.length).toBe(1)
  expect(commands[0]).toMatchObject({ action: 'policy_publish', body: { version: 0, window_seconds: 600, strategy: 'last_touch' } })
})

test('invalid share code returns to public catalog without showing staff login', async ({ page }) => {
  await page.route('**/api/v1/catalog/touches', route => route.fulfill({ status: 422, json: { error: { code: 'CODE_UNAVAILABLE', message: '推广码不存在或已撤销', request_id: 'invalid-code' } } }))
  await page.route('**/api/v1/catalog', route => route.fulfill({ json: { data: { products: [], categories: [], revision: 1 }, request_id: 'catalog' } }))
  await page.goto('/?source=invalid')
  await expect(page.getByText('推广入口暂不可用', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: '返回产品列表', exact: true }).click()
  await expect(page.getByText('暂无已发布产品', { exact: true })).toBeVisible()
  await expect(page.getByLabel('管理员账号', { exact: true })).toHaveCount(0)
})

test('share visits retry with same key while refresh records a distinct visit', async ({ page }) => {
  const keys = new Set<string>()
  await page.route('**/api/v1/catalog/touches', route => {
    expect(route.request().postDataJSON()).toEqual({ code: 'test-code', scene: 'web_share' })
    keys.add(route.request().headers()['idempotency-key'])
    return route.fulfill({ json: { data: { touch_token: 'b'.repeat(48), product: null, task: null, notice: '访问不产生佣金数字' }, request_id: 'visit' } })
  })
  await page.goto('/?source=test-code')
  await expect(page.getByText('Crowd 推广邀请', { exact: true })).toBeVisible()
  expect(keys.size).toBe(1)
  await page.reload()
  await expect(page.getByText('Crowd 推广邀请', { exact: true })).toBeVisible()
  expect(keys.size).toBe(2)
})
