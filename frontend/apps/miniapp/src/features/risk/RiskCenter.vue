<script setup lang="ts">
import { ref, watch } from 'vue'
import { risk, type RiskRecord } from './api'
import { commissions } from '../commissions/api'
const props = defineProps<{ revision: number; run: (work: () => Promise<void>) => Promise<unknown> }>()
const rows = ref<RiskRecord[]>([]); const adjustments = ref<Record<string, unknown>[]>([])
const loading = ref(false), loaded = ref(false), error = ref('')
let loadRevision = 0
async function load() { const stamp = ++loadRevision; loading.value = true; error.value = ''; try { const [nextRows, nextAdjustments] = await Promise.all([risk.myRisk(), commissions.adjustments()]); if (stamp !== loadRevision) return; rows.value = nextRows; adjustments.value = nextAdjustments; loaded.value = true } catch (e) { if (stamp === loadRevision) error.value = e instanceof Error ? e.message : '查询失败'; throw e } finally { if (stamp === loadRevision) loading.value = false } }
watch(() => props.revision, () => { void props.run(load) }, { immediate: true })
</script>
<template><view class="card"><text class="heading">数字调整与受限说明</text><text>数字调减与线下付款不同。风险信号不等于违规结论，可在客服工单选择关联风险单或数字流水提出异议。</text><button @click="run(load)">刷新本人说明</button><view v-for="r in rows" :key="r.id"><text>风险单 {{ r.id }} · {{ r.effective ? '指定操作暂受限' : '核查记录／限制已不生效' }}</text><text>{{ r.scope }} · {{ r.resolution || r.notice }}</text><text v-if="r.expires_at">有效期至 {{ new Date(Number(r.expires_at) * 1000).toLocaleString() }}</text></view><view v-for="r in adjustments" :key="String(r.id)"><text>原任务 {{ r.task_id }} · {{ r.direction === 'restore' ? '恢复' : '调减／待审取消' }} {{ r.amount_cents }} 分 · {{ r.status }}</text><text>原因：{{ r.reason }}；{{ r.outcome }}</text><text v-if="r.before_summary">可申请数字：调整前 {{ (r.before_summary as Record<string, unknown>).available_cents }} 分，调整后 {{ (r.after_summary as Record<string, unknown>)?.available_cents }} 分；不是线下付款金额。</text></view><text v-if="loading">正在查询数字调整与核查记录…</text><text v-if="error">{{ error }}；请重试，不代表没有记录。</text><text v-if="loaded && !loading && !error && !rows.length && !adjustments.length">暂无数字调整或风险核查记录。</text></view></template>
<style scoped>.card{background:white;padding:24rpx;margin:24rpx 0;border-radius:16rpx}text{display:block;margin:14rpx 0;overflow-wrap:anywhere}.heading{font-size:34rpx;font-weight:bold}</style>
