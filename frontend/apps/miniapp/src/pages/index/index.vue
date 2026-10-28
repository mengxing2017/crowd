<script setup lang="ts">
import { tasks } from '../../features/tasks/api'
import { identity } from '../../features/identity/api'
import { governance } from '../../features/governance/api'
import { catalog as catalogApi } from '../../features/catalog/api'
import { attribution } from '../../features/attribution/api'
import WithdrawalCenter from '../../features/withdrawals/WithdrawalCenter.vue'
import TaskCenter from '../../features/tasks/TaskCenter.vue'
import { onMounted, ref } from 'vue'
import { onLoad, onShow, onHide } from '@dcloudio/uni-app'
import { BusinessError, executeCommand, platform } from '../../features/platform/api'
import type { User } from '../../features/identity/api'
import type { Rule } from '../../features/governance/api'
import { promotion, qrUrl, type Promotion } from '../../features/promotion/api'
import type { Receipt } from '../../features/receipts/api'
import type { PublicCatalog, PublicProduct, CatalogNotification } from '../../features/catalog/api'
import type { Claim } from '../../features/tasks/api'
import { support } from '../../features/support/api'
import { download, fileUrl, upload } from '../../features/files/api'
import { setToken } from '../../features/session/api'
const catalog = ref<PublicCatalog | null>(null)
const selected = ref<PublicProduct | null>(null)
const selectedTask = ref('')
const claimConfirmed = ref(false)
const claims = ref<Claim[]>([])
const taskRevision = ref(0)
const notifications = ref<CatalogNotification[]>([])
const touchTokens = ref<string[]>([])
const keyword = ref('')
const categoryFilter = ref('')
const minimum = ref('')
const maximum = ref('')
const sort = ref<'name' | 'commission_asc' | 'commission_desc'>('name')
const fallback = ref('')
const attributionDisputes = ref<{ id: string; claim_id: string; status: string; decision_reason?: string }[]>([])
const disputeClaim = ref('')
const disputeReason = ref('')
const disputeEvidence = ref('')
const me = ref<User | null>(null)
const agreement = ref<Rule | null>(null)
const agreed = ref(false)
const source = ref('')
const nickname = ref('')
const method = ref<'wechat' | 'alipay'>('wechat')
const account = ref('')
const name = ref('')
const confirmed = ref(false)
const file = ref('')
const codes = ref<Promotion[]>([])
const product = ref('')
const task = ref('')
const qr = ref('')
const privateImage = ref('')
const fullAccount = ref('')
const reason = ref('')
const application = ref('')
const cases = ref<{ id: string; kind: string; status: string }[]>([])
const busy = ref(false)
const message = ref('')
const pending = ref<{ action: string; body: Record<string, unknown>; key: string; owner: string } | null>(null)
const ticket = ref('')
let counter = 0
function key() { return `${Date.now()}-${++counter}-${Math.random().toString(36).slice(2)}` }
onLoad(query => { if (typeof query?.source === 'string') { source.value = query.source; void visit(query.source) } })
async function visit(code: string) {
  try {
    const touch = await attribution.touch({ code, scene: 'miniapp_share' }, key())
    touchTokens.value.push(touch.touch_token); fallback.value = ''
    if (me.value) await submit('touch_bind', { touch_token: touch.touch_token })
  } catch (e) { fallback.value = `${e instanceof Error ? e.message : '推广码不可用'}；可返回产品列表继续浏览。` }
}
async function loadCatalog() {
  const next = await catalogApi.catalog({ keyword: keyword.value, category: categoryFilter.value || undefined, min_cents: minimum.value ? Number(minimum.value) : undefined, max_cents: maximum.value ? Number(maximum.value) : undefined, sort: sort.value })
  catalog.value = next
  if (selected.value) selected.value = next.products.find(p => p.id === selected.value?.id) || null
}
function openProduct(product: PublicProduct) { selected.value = product; selectedTask.value = product.content.tasks[0]?.id || ''; claimConfirmed.value = false }
async function claimTask() {
  if (!selected.value) return
  await submit('task_claim', { id: selected.value.id, release_id: selected.value.release_id, task_id: selectedTask.value, confirmed: claimConfirmed.value })
  claimConfirmed.value = false; await loadCatalog()
}
function cents(value: number) { return `${Math.trunc(value / 100)}.${String(value % 100).padStart(2, '0')}` }
const imageOrigin = import.meta.env.VITE_API_ORIGIN || 'http://127.0.0.1:8080'

onHide(hidePrivate)
onShow(() => { void run(loadCatalog) })
onMounted(() => { void run(async () => { await loadCatalog(); agreement.value = await governance.agreements() }) })
let runningCount = 0
async function run(work: () => Promise<void>) { runningCount += 1; busy.value = true; message.value = '处理中…'; try { await work(); message.value = '操作完成' } catch (e) { if (e instanceof BusinessError && e.detail.code === 'SESSION_EXPIRED') clearPrivate(); message.value = e instanceof Error ? e.message : '操作失败，请查询原结果' } finally { runningCount -= 1; busy.value = runningCount > 0 } }
async function refresh() {
  const user = await identity.me(); me.value = user; nickname.value = user.nickname
  attributionDisputes.value = (await attribution.mine()).disputes
  claims.value = await tasks.claims(); notifications.value = await catalogApi.notifications()
  taskRevision.value += 1
  codes.value = await promotion.codes(); cases.value = await support.cases()
}
async function submit(action: string, body: Record<string, unknown>) {
  if (pending.value) throw new Error('请先查询上次提交结果，或使用原请求重试')
  pending.value = { action, body, key: key(), owner: me.value?.id || '' }
  try { const value = await executeCommand(action, body, pending.value.key); pending.value = null; await refresh(); return value } catch (e) { if (e instanceof BusinessError && e.detail.code !== 'SERVICE_UNAVAILABLE') pending.value = null; throw e }
}
async function login() {
  if (source.value && touchTokens.value.length === 0) await visit(source.value)
  const credential = await uni.login({ provider: 'weixin' })
  const session = await identity.login({ code: credential.code, agreed: agreed.value, agreement_id: agreement.value?.id, source: source.value || undefined, touch_token: touchTokens.value[0] })
  if (pending.value && pending.value.owner !== session.user_id) pending.value = null; setToken(session.token); for (const token of touchTokens.value) await executeCommand('touch_bind', { touch_token: token }, key()); await refresh()
}
async function choose(purpose: 'avatar' | 'wechat_receipt') {
  const choice = await uni.chooseImage({ count: 1, sizeType: ['compressed'], sourceType: ['album', 'camera'] })
  const result = await upload(choice.tempFilePaths[0], purpose)
  if (purpose === 'avatar') await submit('profile', { nickname: nickname.value, avatar: result.id, version: me.value?.version })
  else { file.value = result.id; privateImage.value = await download(fileUrl(result.id)) }
}
async function receipt() {
  await submit('receipt', { method: method.value, account: account.value, name: name.value || undefined, file_id: file.value || undefined, confirmed: confirmed.value, version: me.value?.version })
  account.value = ''; name.value = ''; file.value = ''; confirmed.value = false; privateImage.value = ''
}
async function reveal(record: Receipt) {
  const value = await submit('receipt_reveal', { id: record.id, reason: '本人核对收款资料' })
  if (typeof value.account === 'string') fullAccount.value = value.account
  if (typeof value.file_id === 'string') privateImage.value = await download(fileUrl(value.file_id))
}
async function showQr(id: string) { qr.value = await download(qrUrl(id)) }
async function saveQr() { if (!qr.value) return; await uni.saveImageToPhotosAlbum({ filePath: qr.value }) }
async function logout() {
  // Clear local data even if network fails; the server session expires independently.
  try { await executeCommand('logout', {}, key()) } finally {
    clearPrivate(); pending.value = null
  }
}
function clearPrivate() { attributionDisputes.value = []; claims.value = []; notifications.value = []; touchTokens.value = []; hidePrivate(); setToken(''); me.value = null; nickname.value = ''; account.value = ''; name.value = ''; codes.value = []; cases.value = []; qr.value = ''; privateImage.value = ''; fullAccount.value = ''; ticket.value = ''; file.value = ''; confirmed.value = false; reason.value = ''; application.value = '' }
// Authentication is held only in memory. No private response is persisted in platform storage.
function hidePrivate() { if (privateImage.value) uni.getFileSystemManager().unlink({ filePath: privateImage.value, fail: () => undefined }); privateImage.value = ''; fullAccount.value = '' }
function previewReceiptImage() { if (privateImage.value) uni.previewImage({ urls: [privateImage.value] }) }
function copy(value: string) { return uni.setClipboardData({ data: value }) }
</script>
<template>
  <view class="page">
    <text class="title">Crowd 个人中心</text>
    <text class="notice">佣金是业务标记数字，不是系统实存资金。实际付款由平台人员在线下手动完成。</text>
    <text class="status">{{ message }}</text>
    <view v-if="fallback" class="card"><text>{{ fallback }}</text><button @click="selected = null; fallback = ''">返回产品列表</button></view>
    <view class="card">
      <text class="heading">产品目录</text><input v-model="keyword" placeholder="搜索产品名称" /><input v-model="minimum" type="number" placeholder="最低佣金（整数分，可选）" /><input v-model="maximum" type="number" placeholder="最高佣金（整数分，可选）" />
      <picker :range="['名称排序', '佣金从低到高', '佣金从高到低']" @change="sort = (['name', 'commission_asc', 'commission_desc'] as const)[Number($event.detail.value)]"><view>排序：{{ sort === 'name' ? '名称' : sort === 'commission_asc' ? '佣金升序' : '佣金降序' }}</view></picker>
      <view v-if="catalog"><button @click="categoryFilter = ''; run(loadCatalog)">全部分类</button><button v-for="c in catalog.categories" :key="c.id" @click="categoryFilter = c.id; run(loadCatalog)">{{ c.name }}</button></view>
      <button :disabled="busy" @click="run(loadCatalog)">搜索／刷新产品</button><text v-if="!catalog">目录尚未加载，可点击重试。</text><text v-else-if="catalog.products.length === 0">暂无符合条件的已发布产品。</text>
      <view v-for="p in catalog?.products || []" :key="p.id" class="record"><text class="heading">{{ p.content.name }}</text><text>{{ p.content.summary }}</text><text v-for="t in p.content.tasks" :key="t.id">{{ t.name }} · {{ t.price_preview?.kind === 'proportional' && t.price_preview.cap_cents == null ? '按可信成果比例待核定' : `${t.price_preview?.kind === 'proportional' ? '佣金上限' : '佣金标记'} ${cents(t.commission_cents)} 元` }}</text><button @click="openProduct(p)">查看条件与任务</button></view>
    </view>
    <view v-if="selected" class="card"><text class="heading">{{ selected.content.name }}</text><image v-for="img in selected.content.images" :key="img.file_id" :src="`${imageOrigin}/api/v1/catalog/images/${encodeURIComponent(img.file_id)}`" mode="widthFix" @error="message = '图片暂不可用，请阅读文字说明'" /><text v-for="img in selected.content.images" :key="`alt-${img.file_id}`">{{ img.alt }}</text><text>{{ selected.content.details }}</text><text class="notice">关键限制：{{ selected.content.limitations }}</text><text>预计 {{ selected.content.estimated_minutes }} 分钟；适用地区：{{ selected.content.regions.join('、') || '不限' }}；资格：{{ selected.content.qualifications.join('、') || '不限' }}</text>
      <radio-group @change="selectedTask = $event.detail.value; claimConfirmed = false"><label v-for="t in selected.content.tasks" :key="t.id"><radio :value="t.id" :checked="selectedTask === t.id" />{{ t.name }} · {{ t.price_preview?.kind === 'proportional' && t.price_preview.cap_cents == null ? '按比例待核定' : `${t.price_preview?.kind === 'proportional' ? '上限' : '固定'} ${cents(t.commission_cents)} 元` }}</label></radio-group>
      <view v-for="t in selected.content.tasks.filter(t => t.id === selectedTask)" :key="t.id"><text>完成条件：{{ t.completion }}；证据：{{ t.evidence }}；时限：{{ t.submit_seconds }} 秒；{{ t.allow_self ? '允许自推' : '不允许本人自推' }}</text><text v-if="t.template_preview">步骤：{{ t.template_preview.steps.map(s => s.title).join(' → ') }}；失败条件：{{ t.template_preview.failure_conditions }}；审核目标：{{ t.template_preview.review_goal }}；{{ t.template_preview.double_review ? '须独立双人复核' : '人工审核' }}</text><text v-if="t.price_preview?.kind === 'proportional'">{{ t.price_preview.cap_cents == null ? '数字待核定，不显示保证佣金；' : '所示数字为单笔上限；' }}比例 {{ t.price_preview.numerator }}/{{ t.price_preview.denominator }}，{{ t.price_preview.rounding }} 舍入，实际由可信成果基数计算。</text><text>佣金为标记数字，后续审核与人工线下结算不代表自动到账。</text></view>
      <checkbox-group @change="claimConfirmed = $event.detail.value.includes('claim')"><label><checkbox value="claim" :checked="claimConfirmed" />确认当前任务条件、限制和佣金数字</label></checkbox-group><button :disabled="busy || !!pending || !me || !claimConfirmed || !selected.available_tasks.includes(selectedTask)" @click="run(claimTask)">{{ me ? '确认领取并固定快照' : '请先登录后领取' }}</button><button @click="selected = null">返回产品列表</button>
    </view>
    <view v-if="!me" class="card">
      <text class="heading">微信注册与登录</text>
      <view v-if="agreement"><text>服务协议</text><text class="document">{{ agreement.content.service }}</text><text>隐私说明</text><text class="document">{{ agreement.content.privacy }}</text></view>
      <text v-else>协议暂不可用，新用户注册暂不开放。</text><input v-model="source" placeholder="推广来源码（可选）" />
      <checkbox-group @change="agreed = $event.detail.value.includes('agree')"><label><checkbox value="agree" :checked="agreed" />我已阅读并主动同意本版本服务协议与隐私说明</label></checkbox-group>
      <button :disabled="busy" @click="run(login)">微信登录／会话过期后重新登录</button>
      <button :disabled="busy" @click="run(async () => { agreement = await governance.agreements() })">重新加载协议</button>
    </view>
    <template v-else>
      <WithdrawalCenter :key="`withdrawal-${me.id}`" :me="me" :revision="taskRevision" :busy="busy" :blocked="!!pending" :submit="submit" />
      <TaskCenter :key="me.id" :busy="busy" :pending="!!pending" :revision="taskRevision" :submit="submit" :run="run" />
      <view class="card"><text class="heading">已领取任务与历史快照</text><text v-if="claims.length === 0">暂无领取记录；扫码不会占用名额。</text><view v-for="c in claims" :key="c.id" class="record"><text>{{ c.product.name }} · {{ c.task.name }} · {{ c.task.price_preview?.kind === 'proportional' && c.task.price_preview.cap_cents == null ? '按比例待核定' : `${cents(c.task.commission_cents)} 元（${c.task.price_preview?.kind === 'proportional' ? '单笔上限' : '标记数字'}）` }}</text><text>实例：{{ c.id }}；条件：{{ c.task.completion }}；限制：{{ c.product.limitations }}；原分类：{{ c.category.name }}</text><text>提交期限：{{ new Date(c.submit_until * 1000).toLocaleString() }}；已固定领取时规则与推广归属</text><button @click="disputeClaim = c.id">申诉此任务归因</button></view><text>新版任务可在下方“我的任务执行与提交”处理；旧版领取快照不变。</text><input v-model="disputeClaim" placeholder="申诉任务实例编号" /><input v-model="disputeReason" placeholder="申诉理由" /><input v-model="disputeEvidence" placeholder="触点证据说明" /><button :disabled="busy || !!pending || !disputeClaim || !disputeReason || !disputeEvidence" @click="run(async () => { await submit('attribution_dispute', { claim_id: disputeClaim, reason: disputeReason, evidence: disputeEvidence }) })">提交归因申诉（不直接改数字）</button><view v-for="d in attributionDisputes" :key="d.id"><text>申诉 {{ d.claim_id }}：{{ d.status === 'pending' ? '待人工裁定' : d.status === 'uphold' ? '维持原归属' : '待关联数字调整核查' }} {{ d.decision_reason }}</text></view></view>
      <view v-if="notifications.length" class="card"><text class="heading">产品状态通知</text><text v-for="(n, i) in notifications" :key="i">{{ n.product_id }} · {{ n.kind === 'product_offline' ? '已下架，存量按快照继续' : n.kind === 'product_archive' ? '已归档，历史记录保留' : n.kind === 'scheduled_live' ? '定时发布已生效' : n.kind === 'expired_or_unavailable' ? '已到期或不可用，存量按快照继续' : '发布审核状态已更新' }}</text></view>
      <view class="card">
        <text class="heading">{{ me.nickname }} · {{ me.status }}</text><text>用户编号：{{ me.id }}</text>
        <view v-for="(restriction, index) in me.restrictions" :key="index"><text>限制范围：{{ restriction.scope }}；{{ restriction.reason }}；申诉：{{ restriction.appeal }}</text></view>
        <button :disabled="busy" @click="run(refresh)">刷新资料与处理结果</button>
        <button :disabled="busy" @click="run(logout)">退出并清理敏感缓存</button>
      </view>
      <view class="card">
        <text class="heading">基础资料</text><input v-model="nickname" placeholder="昵称" />
        <button :disabled="busy || !!pending" @click="run(async () => { await submit('profile', { nickname, avatar: me?.avatar, version: me?.version }) })">保存昵称</button>
        <button :disabled="busy || !!pending" @click="run(() => choose('avatar'))">上传头像</button>
        <text>浏览、领取及完成任务不默认要求先填写收款资料。手机号绑定尚未启用。</text>
      </view>
      <view class="card">
        <text class="heading">本人收款资料</text><text>微信仅上传本人收款二维码；支付宝仅填写账户。上传与格式检查不代表官方账户验证。历史申请固定原资料版本。</text>
        <radio-group @change="method = $event.detail.value as 'wechat' | 'alipay'"><label><radio value="wechat" :checked="method === 'wechat'" />微信收款码</label><label><radio value="alipay" :checked="method === 'alipay'" />支付宝账户</label></radio-group>
        <button v-if="method === 'wechat'" :disabled="busy" @click="run(() => choose('wechat_receipt'))">选择本人收款码图片</button>
        <view v-else><input v-model="account" placeholder="支付宝收款账户" /><input v-model="name" placeholder="收款姓名（可选）" /><text>账户格式及姓名要求以已发布政策为准；未配置政策时仅存草稿。格式通过不代表账户存在，付款前仍需人工核对。</text></view>
        <checkbox-group @change="confirmed = $event.detail.value.includes('confirmed')"><label><checkbox value="confirmed" :checked="confirmed" />确认此资料正确、可供本人线下收款（声明版本 receipt-self-v1）</label></checkbox-group>
        <button :disabled="busy || !!pending || !confirmed || (method === 'wechat' && !file)" @click="run(receipt)">保存为新的资料版本</button>
        <text v-if="me.receipts.length === 0">尚未填写收款资料。</text>
        <view v-for="record in me.receipts" :key="record.id" class="record"><text>{{ record.method || '停用记录' }} · {{ record.status }} · V{{ record.version }} {{ record.account }} {{ record.is_default ? '默认新申请资料' : '' }}</text>
          <template v-if="record.method"><button :disabled="busy || !!pending" @click="run(() => reveal(record))">本人查看完整资料（留痕）</button><button :disabled="busy || !!pending" @click="run(async () => { await submit('receipt_disable', { id: record.id, version: me?.version }) })">停用并保留历史</button><button v-if="record.method === 'wechat' && me.receipts.some(r => r.disables === record.id) && !me.receipts.some(r => r.cleans === record.id)" :disabled="busy || !!pending" @click="run(async () => { await submit('receipt_cleanup', { id: record.id, version: me?.version, reason: '本人请求按已发布保留政策清理停用且无引用的码图' }) })">按保留政策清理无引用旧图</button></template>
        </view>
        <text v-if="privateImage">请检查码图清晰、完整且无遮挡；点击图片放大核对，不代表官方收款认证。</text><image v-if="privateImage" @click="previewReceiptImage" :src="privateImage" mode="widthFix" /><text v-if="fullAccount">{{ fullAccount }}</text><button v-if="privateImage || fullAccount" @click="hidePrivate">关闭敏感资料</button>
      </view>
      <view class="card">
        <text class="heading">个人独立推广码</text><text>推广码与收款码不同；扫码不代表完成任务，也不产生可申请佣金。</text>
        <input v-model="product" placeholder="定向产品编号（可选）" /><input v-model="task" placeholder="定向任务编号（可选）" /><button :disabled="busy || !!pending || !product" @click="run(async () => { await submit('code_create', { product, task: task || undefined }) })">生成产品／任务定向码</button><text v-if="codes.length === 0">暂未生成，可使用重试按钮恢复。</text><button v-if="codes.length === 0" :disabled="busy || !!pending" @click="run(async () => { await submit('code_create', {}) })">重试生成主码</button>
        <view v-for="promotion in codes" :key="promotion.id" class="record"><text>{{ promotion.id }} · {{ promotion.code.active ? '有效' : '已撤销' }}</text><template v-if="promotion.code.active"><button @click="copy(promotion.id)">复制推广码</button><button :disabled="busy" @click="run(() => showQr(promotion.id))">显示推广二维码</button></template></view>
        <image v-if="qr" :src="qr" mode="widthFix" /><button v-if="qr" :disabled="busy" @click="run(saveQr)">保存二维码到相册</button><text>相册权限拒绝后仍可查看与复制推广码。定向入口仅支持已发布目录中的有效产品与任务。</text>
      </view>
      <view class="card">
        <text class="heading">资料纠错与账号申请</text><input v-model="reason" placeholder="申请理由（必填）" /><input v-model="application" placeholder="受影响申请编号（纠错时填写）" />
        <button :disabled="busy || !!pending || !reason" @click="run(async () => { await submit('correction', { reason, application_id: application }) })">反馈资料有误</button>
        <button :disabled="busy || !!pending || !reason" @click="run(async () => { await submit('merge', { reason }) })">申请人工核验重复身份</button>
        <button :disabled="busy || !!pending || !reason" @click="run(async () => { await submit('closure', { reason }) })">申请注销并核查未结事项</button>
        <text>申请不自动合并身份、删除资料、释放占用或更改付款账户。注销需完成关联业务核查。</text><text v-if="cases.length === 0">暂无申请。</text><text v-for="(item, index) in cases" :key="index" class="document">{{ item.kind === 'closure' ? '注销申请' : item.kind === 'merge' ? '身份核验申请' : '资料纠错申请' }} · {{ item.status === 'pending_dependency_check' ? '待关联业务核查' : '待人工核实' }} · {{ item.id }}</text>
      </view>
      <view class="card"><text class="heading">后台登录</text><button :disabled="busy || !!pending" @click="run(async () => { const result = await submit('web_ticket', {}); ticket = String(result.ticket); await copy(ticket) })">生成并复制一次性后台票据</button><text v-if="ticket">60 秒内在管理端使用，使用后立即失效。请勿转交他人。</text></view>
      <view v-if="pending" class="card"><text>上次提交结果未确认，请先查询原结果。重试使用原请求键及原内容。</text><button :disabled="busy" @click="run(async () => { await platform.commandResult(pending!.key); pending = null; await refresh() })">查询原结果</button><button :disabled="busy" @click="run(async () => { if (pending) { await executeCommand(pending.action, pending.body, pending.key); pending = null; await refresh() } })">重试原提交</button></view>
    </template>
  </view>
</template>
<style scoped>
.page { padding: 32rpx; background: #f3f7f6; color: #172f35; min-height: 100vh; } text { display: block; margin: 14rpx 0; overflow-wrap: anywhere; } .title { font-size: 44rpx; font-weight: bold; } .heading { font-size: 34rpx; font-weight: bold; } .card { background: white; border-radius: 16rpx; padding: 24rpx; margin: 24rpx 0; } .notice, .status { color: #356452; } .document { white-space: pre-wrap; padding: 16rpx; background: #f4f6f6; } input { border: 1px solid #b3c4bd; padding: 18rpx; margin: 20rpx 0; } button { margin: 16rpx 0; } .record { border-top: 1px solid #dfebe4; padding: 16rpx 0; } image { width: 100%; }
</style>
