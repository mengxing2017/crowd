<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { onLoad } from '@dcloudio/uni-app'
import { BusinessError, type User, type Rule, type Promotion, type Receipt } from '../../features/community/api'
import { community, download, fileUrl, qrUrl, setToken, upload } from '../../features/community/api/client'
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
onLoad(query => { if (typeof query?.source === 'string') source.value = query.source })
onMounted(() => { void run(async () => { agreement.value = await community.agreements() }) })
async function run(work: () => Promise<void>) { busy.value = true; message.value = '处理中…'; try { await work(); message.value = '操作完成' } catch (e) { if (e instanceof BusinessError && e.detail.code === 'SESSION_EXPIRED') clearPrivate(); message.value = e instanceof Error ? e.message : '操作失败，请查询原结果' } finally { busy.value = false } }
async function refresh() {
  const user = await community.read<User>('me'); me.value = user; nickname.value = user.nickname
  codes.value = await community.read<Promotion[]>('codes'); cases.value = await community.read<{ id: string; kind: string; status: string }[]>('cases')
}
async function submit(action: string, body: Record<string, unknown>) {
  pending.value = { action, body, key: key(), owner: me.value?.id || '' }
  try { const value = await community.command(action, body, pending.value.key); pending.value = null; await refresh(); return value } catch (e) { if (e instanceof BusinessError && e.detail.code !== 'SERVICE_UNAVAILABLE') pending.value = null; throw e }
}
async function login() {
  const credential = await uni.login({ provider: 'weixin' })
  const session = await community.login({ code: credential.code, agreed: agreed.value, agreement_id: agreement.value?.id, source: source.value || undefined })
  if (pending.value && pending.value.owner !== session.user_id) pending.value = null; setToken(session.token); await refresh()
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
  try { await community.command('logout', {}, key()) } finally {
    clearPrivate(); pending.value = null
  }
}
function clearPrivate() { hidePrivate(); setToken(''); me.value = null; nickname.value = ''; account.value = ''; name.value = ''; codes.value = []; cases.value = []; qr.value = ''; privateImage.value = ''; fullAccount.value = ''; ticket.value = ''; file.value = ''; confirmed.value = false; reason.value = ''; application.value = '' }
// Authentication is held only in memory. No private response is persisted in platform storage.
function hidePrivate() { if (privateImage.value) uni.getFileSystemManager().unlink({ filePath: privateImage.value, fail: () => undefined }); privateImage.value = ''; fullAccount.value = '' }
function copy(value: string) { return uni.setClipboardData({ data: value }) }
</script>
<template>
  <view class="page">
    <text class="title">Crowd 个人中心</text>
    <text class="notice">佣金是业务标记数字，不是系统实存资金。实际付款由平台人员在线下手动完成。</text>
    <text class="status">{{ message }}</text>
    <view v-if="!me" class="card">
      <text class="heading">微信注册与登录</text>
      <view v-if="agreement"><text>服务协议</text><text class="document">{{ agreement.content.service }}</text><text>隐私说明</text><text class="document">{{ agreement.content.privacy }}</text></view>
      <text v-else>协议暂不可用，新用户注册暂不开放。</text><input v-model="source" placeholder="推广来源码（可选）" />
      <checkbox-group @change="agreed = $event.detail.value.includes('agree')"><label><checkbox value="agree" :checked="agreed" />我已阅读并主动同意本版本服务协议与隐私说明</label></checkbox-group>
      <button :disabled="busy" @click="run(login)">微信登录／会话过期后重新登录</button>
      <button :disabled="busy" @click="run(async () => { agreement = await community.agreements() })">重新加载协议</button>
    </view>
    <template v-else>
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
        <view v-else><input v-model="account" placeholder="支付宝收款账户" /><input v-model="name" placeholder="收款姓名（可选）" /><text>支持账户形式待业务确认，当前保存为草稿，不宣称可申请使用。</text></view>
        <checkbox-group @change="confirmed = $event.detail.value.includes('confirmed')"><label><checkbox value="confirmed" :checked="confirmed" />确认此资料可供本人线下收款</label></checkbox-group>
        <button :disabled="busy || !!pending || !confirmed || (method === 'wechat' && !file)" @click="run(receipt)">保存为新的资料版本</button>
        <text v-if="me.receipts.length === 0">尚未填写收款资料。</text>
        <view v-for="record in me.receipts" :key="record.id" class="record"><text>{{ record.method || '停用记录' }} · {{ record.status }} · V{{ record.version }} {{ record.account }}</text>
          <template v-if="record.method"><button :disabled="busy || !!pending" @click="run(() => reveal(record))">本人查看完整资料（留痕）</button><button :disabled="busy || !!pending" @click="run(async () => { await submit('receipt_disable', { id: record.id, version: me?.version }) })">停用并保留历史</button></template>
        </view>
        <image v-if="privateImage" :src="privateImage" mode="widthFix" /><text v-if="fullAccount">{{ fullAccount }}</text><button v-if="privateImage || fullAccount" @click="hidePrivate">关闭敏感资料</button>
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
      <view v-if="pending" class="card"><text>上次提交结果未确认，请先查询原结果。重试使用原请求键及原内容。</text><button :disabled="busy" @click="run(async () => { await community.read('command_result', pending?.key); pending = null; await refresh() })">查询原结果</button><button :disabled="busy" @click="run(async () => { if (pending) { await community.command(pending.action, pending.body, pending.key); pending = null; await refresh() } })">重试原提交</button></view>
    </template>
  </view>
</template>
<style scoped>
.page { padding: 32rpx; background: #f3f7f6; color: #172f35; min-height: 100vh; } text { display: block; margin: 14rpx 0; overflow-wrap: anywhere; } .title { font-size: 44rpx; font-weight: bold; } .heading { font-size: 34rpx; font-weight: bold; } .card { background: white; border-radius: 16rpx; padding: 24rpx; margin: 24rpx 0; } .notice, .status { color: #356452; } .document { white-space: pre-wrap; padding: 16rpx; background: #f4f6f6; } input { border: 1px solid #b3c4bd; padding: 18rpx; margin: 20rpx 0; } button { margin: 16rpx 0; } .record { border-top: 1px solid #dfebe4; padding: 16rpx 0; } image { width: 100%; }
</style>
