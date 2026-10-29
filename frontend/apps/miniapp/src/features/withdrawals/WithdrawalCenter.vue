<script setup lang="ts">
import { onHide } from '@dcloudio/uni-app'
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { BusinessError } from '../platform/api'
import { withdrawals, withdrawalStatusNames, withdrawalStatusLabel, type WithdrawalApplication, type WithdrawalPreview, type WithdrawalReveal, type WithdrawalRule } from './api'
import { commissions, type CommissionSource, type CommissionSummary, type CommissionEntry } from '../commissions/api'
import type { User } from '../identity/api'
import { fileUrl, download } from '../files/api'
const props = defineProps<{ me: User; revision: number; busy: boolean; blocked: boolean; submit: (action: string, body: Record<string, unknown>) => Promise<Record<string, unknown>> }>()
const summary = ref<CommissionSummary>()
const entries = ref<CommissionEntry[]>([])
const cursor = ref<number | null>(null)
const applications = ref<WithdrawalApplication[]>([])
const rule = ref<WithdrawalRule>()
const amount = ref('')
const receiptId = ref('')
const preview = ref<WithdrawalPreview>()
const confirmed = ref(false)
const reason = ref('')
const recordId = ref('')
const sourceDetail = ref<CommissionSource>()
const message = ref('')
const loaded = ref(false), loadError = ref(''), fieldError = ref('')
const working = ref(false)
const image = ref('')
const reveal = ref<WithdrawalReveal>()
const originalAccount = ref('')
const originalImage = ref('')
function clearSensitive() { for (const path of [image.value, originalImage.value]) if (path) uni.getFileSystemManager().unlink({ filePath: path, fail: () => undefined }); image.value = ''; originalImage.value = ''; originalAccount.value = ''; reveal.value = undefined; confirmed.value = false }
onHide(clearSensitive)
onUnmounted(clearSensitive)
function format(n: number) { const s = String(Math.abs(n)); return `${n < 0 ? '-' : ''}${s.length > 2 ? s.slice(0, -2) : '0'}.${s.slice(-2).padStart(2, '0')}` }
// Parse decimal text exactly; reject excess precision instead of silently rounding.
function parse(value: string): number { if (!/^(0|[1-9]\d*)(\.\d{1,2})?$/.test(value)) throw new Error('请输入正数，最多两位小数'); const [whole, fraction = ''] = value.split('.'); const n = Number(`${whole}${fraction.padEnd(2, '0')}`); if (!Number.isSafeInteger(n) || n <= 0) throw new Error('申请数字超出范围'); return n }
async function run(work: () => Promise<void>) { if (working.value || props.busy || props.blocked) return; working.value = true; message.value = '处理中…'; try { await work(); message.value = '操作完成' } catch (e) { if (e instanceof BusinessError && ['INSUFFICIENT_AVAILABLE', 'VERSION_CONFLICT'].includes(e.detail.code)) await load(); message.value = e instanceof Error ? e.message : '操作失败，请查询原提交结果' } finally { working.value = false } }
async function load() { loadError.value = ''; try { const [s, page, apps] = await Promise.all([commissions.summary(), commissions.entries(`${props.me.id}|0`), withdrawals.mine()]); summary.value = s; entries.value = page.items; cursor.value = page.next_cursor; applications.value = apps; try { const result = await withdrawals.rules(); rule.value = Array.isArray(result) ? result.find(r => r.status === 'published') : result } catch (e) { rule.value = undefined; message.value = e instanceof Error ? e.message : '规则尚未配置' } loaded.value = true } catch (e) { loadError.value = e instanceof Error ? e.message : '加载失败'; throw e } }
onMounted(() => { void run(load) })
watch(() => props.revision, () => { void load().catch(e => { message.value = e instanceof Error ? e.message : '查询失败，请重试' }) })
watch([amount, receiptId], () => { preview.value = undefined; confirmed.value = false })
function parseAmount() { fieldError.value = ''; try { return parse(amount.value) } catch (e) { fieldError.value = e instanceof Error ? e.message : '申请数字格式错误'; throw e } }
function body() { const receipt = props.me.receipts.find(r => r.id === receiptId.value); if (!receipt) throw new Error('请选择本人收款资料'); return { amount_cents: parseAmount(), receipt_id: receipt.id, receipt_version: receipt.version, summary_version: summary.value?.version } }
async function prepare() { preview.value = await props.submit('withdrawal_preview', body()) as unknown as WithdrawalPreview; confirmed.value = false }
async function apply() { if (!preview.value || !confirmed.value) throw new Error('请先预览并主动确认原收款资料和规则'); await props.submit('withdrawal_apply', { ...body(), preview_id: preview.value.id, confirmed: true }); preview.value = undefined; amount.value = ''; await load() }
async function cancel(app: WithdrawalApplication) { if (!reason.value.trim()) throw new Error('请填写撤销原因'); await props.submit('withdrawal_cancel', { id: app.id, version: app.version, reason: reason.value }); await load() }
async function show(app: WithdrawalApplication) { if (!reason.value.trim()) throw new Error('请填写查看资料用途'); reveal.value = await props.submit('withdrawal_receipt_reveal', { id: app.id, reason: reason.value }) as unknown as WithdrawalReveal }
async function showImage(id: string) { image.value = await download(fileUrl(id, reason.value)); uni.previewImage({ urls: [image.value] }) }
async function complaint(appId?: string) { if (!reason.value.trim()) throw new Error('请填写异议说明'); await props.submit('correction', { reason: reason.value, application_id: appId || recordId.value, record_id: recordId.value }); message.value = '已提交核查；系统数字纠错与线下付款核实分别处理' }
</script>
<template>
 <view class="card">
  <text class="title">佣金数字与提现申请</text><text v-if="loadError" role="alert">{{ loadError }}；请重试，不代表没有佣金记录。</text><text v-else-if="!loaded">正在首次加载数字与申请…</text><text v-if="fieldError" role="alert">申请数字：{{ fieldError }}</text><text>仅为平台业务数字，非存款余额。付款由人员线下处理，无自动到账承诺。</text>
  <button :disabled="working || busy || blocked" @click="run(load)">刷新数字、流水和申请</button><text>{{ message }}</text>
  <view v-if="summary"><text v-if="summary.gross_confirmed_cents !== undefined && summary.adjustment_cents !== undefined">累计确认 {{ format(summary.gross_confirmed_cents) }} 元 / 净调整 {{ format(summary.adjustment_cents) }} 元</text><text>待审 {{ format(summary.pending_cents) }} 元 / 可申请 {{ format(summary.available_cents) }} 元</text><text>申请占用 {{ format(summary.held_cents) }} 元 / 争议暂缓 {{ format(summary.disputed_cents) }} 元 / 已登记结算 {{ format(summary.settled_cents) }} 元</text><text v-if="!summary.consistent">数字异常，暂停申请，请关联流水提交核查。</text></view>
  <view v-if="rule"><text>当前规则版本 {{ rule.version }}：最低 {{ format(rule.min_cents) }} 元，最高 {{ format(rule.max_cents) }} 元；每日 {{ rule.daily_count }} 次、{{ format(rule.daily_cents) }} 元，累计上限 {{ format(rule.lifetime_cents) }} 元；无额外扣减。</text><text>按北京时间统计提交次数，包含已撤销及驳回申请。</text>
   <input v-model="amount" type="digit" aria-label="申请金额（元，最多两位小数）" placeholder="申请金额（元，最多两位小数）" />
   <view v-for="r in me.receipts.filter(r => r.method && r.status === 'confirmed' && !me.receipts.some(x => x.disables === r.id))" :key="r.id"><button :disabled="busy || blocked || working || !rule.methods.includes(r.method!)" @click="receiptId = r.id">{{ receiptId === r.id ? '已选择：' : '选择：' }}{{ r.method === 'wechat' ? '微信收款二维码' : '支付宝账户' }} {{ r.account || '' }}（版本 {{ r.version }}）{{ r.is_default ? '默认资料' : '' }}</button></view>
   <button :disabled="working || busy || blocked || !summary?.consistent" @click="run(prepare)">预览申请</button>
   <view v-if="preview"><text>确认申请 {{ format(preview.amount_cents) }} 元；收款资料 {{ preview.receipt_id }}，版本 {{ preview.receipt_version }}。{{ preview.notice }}</text><button @click="run(async () => { const r = await props.submit('receipt_reveal', { id: receiptId, reason: '提交提现前本人核对收款资料' }); originalAccount = String(r.account || ''); if (typeof r.file_id === 'string') originalImage = await download(fileUrl(r.file_id)) })">核对本人原收款资料</button><text v-if="originalAccount">{{ originalAccount }}</text><image v-if="originalImage" :src="originalImage" mode="widthFix" /><label><checkbox :checked="confirmed" @click="confirmed = !confirmed" />已查看本人收款资料，确认数字及人工处理规则</label><button :disabled="working || busy || blocked || !confirmed" :aria-label="`确认申请 ${amount} 元业务数字，仅占用，不自动付款`" @click="run(apply)">确认提交申请</button></view>
  </view><text v-else>提现规则尚未发布，暂不可申请。</text>
  <input v-model="reason" aria-label="撤销、查看资料或异议核查说明" placeholder="撤销、查看资料或异议核查说明" /><input v-model="recordId" aria-label="异议关联流水编号（选填）" placeholder="异议关联流水编号（选填）" /><button :disabled="working || busy || blocked" @click="run(() => complaint())">关联数字记录提交核查</button>
  <view v-for="app in applications" :key="app.id" class="card"><text>{{ app.id }} / {{ format(app.amount_cents) }} 元 / {{ withdrawalStatusLabel(app) }}</text><text v-if="app.source_disputed || app.adjustment_review_required">来源数字争议，暂停继续人工处理并核查；原占用保留。</text><text>收款快照版本 {{ app.receipt_snapshot.version }}；后续修改资料不影响此申请。</text><view v-if="app.registration"><text>人工登记参考 {{ format(app.registration.amount_cents) }} 元；线下付款时间 {{ app.registration.paid_at }}；系统登记时间 {{ app.registration.recorded_at }}。非官方到账证明。</text></view><button v-if="['pending_review', 'awaiting_second_review', 'ready'].includes(app.status) && !app.handler" :disabled="working || busy || blocked" @click="run(() => cancel(app))">撤销并释放本单占用</button><button :disabled="working || busy || blocked" @click="run(() => show(app))">查看原资料及人工凭证</button><button :disabled="working || busy || blocked" @click="run(() => complaint(app.id))">对此申请提出异议</button><view v-for="(event, index) in app.events" :key="index"><text>{{ event.action }} · {{ event.reason }}</text><text v-if="event.handoff">交接 {{ event.handoff.from }} → {{ event.handoff.to }}；{{ event.handoff.paid_status === 'unpaid' ? '未付款' : event.handoff.paid_status === 'paid' ? '已线下付款' : '结果待核实' }}</text></view></view>
  <view v-if="reveal"><text>{{ reveal.receipt_snapshot?.account || '' }}</text><button v-if="reveal.receipt_snapshot?.file_id" @click="run(() => showImage(reveal!.receipt_snapshot.file_id!))">查看原收款码</button><button v-if="reveal.registration?.voucher_file_id" @click="run(() => showImage(reveal!.registration!.voucher_file_id!))">查看人工登记凭证</button></view>
  <text>数字变更流水（整数分）</text><view v-for="entry in entries" :key="entry.id"><text>{{ entry.id }} / {{ entry.kind }} / {{ entry.amount_cents }} 分；来源 {{ entry.source }}</text><button :disabled="working || busy || blocked" @click="run(async () => { sourceDetail = await commissions.source(me.id, entry.source); recordId = entry.id })">查看数字来源及关联申请</button></view><button v-if="cursor !== null" :disabled="working || busy || blocked" @click="run(async () => { const page = await commissions.entries(`${me.id}|${cursor}`); entries.push(...page.items); cursor = page.next_cursor })">加载更多流水</button>
  <view v-if="sourceDetail"><text>数字来源 {{ sourceDetail.source }}；任务状态 {{ sourceDetail.task?.status || '关联业务记录' }}</text><view v-for="a in sourceDetail.applications" :key="a.id"><text>{{ a.id }} / {{ withdrawalStatusNames[a.status] || a.status }} / {{ a.amount_cents }} 分</text></view></view>
 </view>
</template>
