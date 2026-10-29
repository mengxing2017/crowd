<script setup lang="ts">
import { ref, watch } from 'vue'
import { support } from './api'
import { tasks } from '../tasks/api'
import { withdrawals } from '../withdrawals/api'
import { commissions } from '../commissions/api'
import { risk } from '../risk/api'
import { upload, download, fileUrl } from '../files/api'
const props = defineProps<{ revision: number; busy: boolean; blocked: boolean; submit: (action: string, body: Record<string, unknown>) => Promise<Record<string, unknown>>; run: (work: () => Promise<void>) => Promise<unknown> }>()
type Row = Record<string, unknown>
const rows = ref<Row[]>([]); const category = ref('task'); const description = ref(''); const evidence = ref(''); const related = ref(''); const choices = ref<{ id: string; label: string }[]>([]); const policy = ref<Row>(); const reply = ref(''); const selected = ref<Row>(); const attachment = ref(''); const loaded = ref(false)
const categories = [{ id: 'registration', name: '注册与身份' }, { id: 'promotion', name: '推广码' }, { id: 'task', name: '任务咨询' }, { id: 'commission', name: '佣金数字' }, { id: 'withdrawal', name: '提现与人工结算' }, { id: 'appeal', name: '审核驳回申诉' }, { id: 'risk', name: '风险限制申诉' }]
async function load() { rows.value = await support.tickets(); policy.value = await support.supportPolicy(); loaded.value = true }
async function loadChoices() {
 choices.value = []; related.value = ''
 if (category.value === 'withdrawal') choices.value = (await withdrawals.mine()).map(r => ({ id: r.id, label: `${r.id} · ${r.status}` }))
 else if (category.value === 'task' || category.value === 'appeal') choices.value = (await tasks.mine()).items.filter(t => category.value !== 'appeal' || t.status === 'rejected').map(t => ({ id: t.id, label: `${t.id} · ${t.status}` }))
 else if (category.value === 'commission') choices.value = (await commissions.entries()).items.map(e => ({ id: e.id, label: `${e.id} · ${e.amount_cents}分` }))
 else if (category.value === 'risk') choices.value = (await risk.myRisk()).map(r => ({ id: r.id, label: `${r.business_id} · ${r.status}` }))
}
async function create() { await props.submit('ticket_create', { category: category.value, description: description.value, related_id: related.value, evidence: evidence.value }); description.value = ''; await load() }
async function send() { if (!selected.value) return; await props.submit('ticket_reply', { id: selected.value.id, version: selected.value.version, body: reply.value, internal: false, attachments: attachment.value ? [attachment.value] : [], reason: '本人补充工单说明' }); reply.value = ''; attachment.value = ''; await load(); selected.value = rows.value.find(t => t.id === selected.value?.id) }
async function choose() { const picked = await new Promise<string>((resolve, reject) => uni.chooseImage({ count: 1, success: r => resolve(r.tempFilePaths[0]), fail: () => reject(new Error('未选择附件，仍可提交文本说明')) })); attachment.value = (await upload(picked, 'support_attachment')).id }
async function viewAttachment(id: string) { const path = await download(fileUrl(id, '本人查看工单可见附件')); uni.previewImage({ urls: [path] }) }
watch(() => props.revision, () => { void props.run(load) }, { immediate: true })
</script>
<template><view class="card"><text class="heading">客服工单与申诉</text><text>只可关联本人单据。申诉不自动增加佣金；付款结果待核实时不保证线下已收款。</text><button :disabled="busy || blocked" @click="run(load)">刷新本人工单</button><text v-if="policy?.configured">申诉期限 {{ policy.appeal_seconds }} 秒；最多 {{ policy.max_appeals }} 次；结案重开期限 {{ policy.reopen_seconds }} 秒。</text><text v-else>尚未配置正式申诉次数和期限，可先提交普通咨询。</text>
<picker :range="categories" range-key="name" @change="category = categories[Number($event.detail.value)].id; run(loadChoices)"><view>问题分类：{{ categories.find(c => c.id === category)?.name }}（点击选择）</view></picker><button :disabled="busy || blocked" @click="run(loadChoices)">加载本人可关联单据</button><picker v-if="choices.length" :range="choices" range-key="label" @change="related = choices[Number($event.detail.value)].id"><view>关联单据：{{ related || '请选择' }}</view></picker><textarea v-model="description" placeholder="问题描述与期望处理" /><textarea v-model="evidence" placeholder="申诉理由与补充证据说明" /><button :disabled="busy || blocked || !description" @click="run(create)">提交分类工单／申诉</button>
<text v-if="loaded && !rows.length">暂无工单。</text><view v-for="t in rows" :key="String(t.id)"><text>{{ t.id }} · {{ t.category }} · {{ t.status }}</text><text>{{ t.description }}；{{ t.conclusion }}</text><button @click="selected = t">查看回复与补充</button><button v-if="t.status === 'closed'" :disabled="busy || blocked" @click="run(async () => { await submit('ticket_reopen', { id: t.id, version: t.version, reason: '本人反馈问题仍未解决' }); await load() })">按政策重新打开</button></view>
<view v-if="selected"><text>工单 {{ selected.id }}</text><view v-for="r in (selected.replies as Row[])" :key="String(r.id)"><text>{{ r.body }}</text><button v-for="id in (r.attachments as string[])" :key="id" @click="run(() => viewAttachment(id))">查看本工单附件</button></view><textarea v-model="reply" placeholder="补充说明" /><button :disabled="busy || blocked" @click="run(choose)">上传受控图片附件</button><text v-if="attachment">本次附件已上传</text><button :disabled="busy || blocked || !reply" @click="run(send)">补充到原工单</button></view>
</view></template>
<style scoped>.card{background:white;padding:24rpx;margin:24rpx 0;border-radius:16rpx}text{display:block;margin:14rpx 0;overflow-wrap:anywhere}.heading{font-size:34rpx;font-weight:bold}textarea{border:1px solid #b3c4bd;margin:16rpx 0;padding:12rpx;width:90%}</style>
