<script setup lang="ts">
import { ref, watch } from 'vue'
import { notifications, type NotificationsRecord } from './api'
const props = defineProps<{ revision: number; busy: boolean; blocked: boolean; submit: (action: string, body: Record<string, unknown>) => Promise<Record<string, unknown>>; run: (work: () => Promise<void>) => Promise<unknown> }>()
const rows = ref<NotificationsRecord[]>([])
const loaded = ref(false)
const loadError = ref('')
async function load() { loadError.value = ''; try { rows.value = await notifications.messages(); loaded.value = true } catch (error) { loadError.value = error instanceof Error ? error.message : '消息查询失败'; throw error } }
async function mark(row: NotificationsRecord) { await props.submit('notification_read', { id: row.id, version: row.version }); await load().catch(() => undefined) }
function jump(row: NotificationsRecord) { uni.showModal({ title: '查看当前业务', content: `请在本页${({ tasks: '任务执行', commissions: '佣金数字', withdrawals: '提现与人工处理', support: '客服工单', risk: '受限说明' } as Record<string,string>)[String(row.target)] || '业务'}中查看编号 ${String(row.business_id)}。当前状态：${String(row.current_status || row.status)}。`, showCancel: false }) }
watch(() => props.revision, () => { void props.run(load) }, { immediate: true })
</script>
<template><view class="card"><text class="heading">站内消息中心</text><text>通知关联真实单据，以当前状态为准；佣金变化不代表真实资金收付。</text><button :disabled="busy || blocked" @click="run(load)">刷新本人消息</button><text v-if="loadError">{{ loadError }}；请刷新查询，不代表没有消息。</text><text v-if="loaded && !loadError && !rows.length">暂无站内消息。</text><view v-for="row in rows.filter(r => r.kind === 'message')" :key="row.id"><text>{{ row.business_id }} · {{ row.status }} · 当前 {{ row.current_status || row.status }}</text><text>{{ row.body }}</text><text v-if="row.reason && (row.reason as Record<string, unknown>).amount_cents != null">数字变更 {{ (row.reason as Record<string, unknown>).amount_cents }} 分（非真实资金收付）</text><text>{{ row.notice }}</text><button @click="jump(row)">查看当前业务说明</button><button v-if="!row.read_at" :disabled="busy || blocked" @click="run(() => mark(row))">标记已读</button></view><text>外部订阅未配置，不影响站内查询。</text><button :disabled="busy || blocked" @click="run(async () => { await submit('notification_consent', { template: 'business_updates', version: rows.find(r => r.kind === 'consent' && r.template === 'business_updates')?.version || 0, granted: false, scene: 'message_center' }) })">暂不订阅外部消息</button></view></template>
<style scoped>.card{background:white;padding:24rpx;margin:24rpx 0;border-radius:16rpx}text{display:block;margin:14rpx 0;overflow-wrap:anywhere}.heading{font-size:34rpx;font-weight:bold}</style>
