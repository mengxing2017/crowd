<script setup lang="ts">
import { onUnmounted, ref, watch } from 'vue'
import { onSessionExpired } from '../session/api'
import { usePrivateScope } from '../../shared/use-private-scope'
import { privacy, type OperationRecord } from './api'
const props = defineProps<{ revision: number; busy: boolean; blocked: boolean; submit: (action: string, body: Record<string, unknown>) => Promise<Record<string, unknown>>; run: (work: () => Promise<void>) => Promise<void> }>()
const policy = ref<OperationRecord | null>(null), requests = ref<OperationRecord[]>([]), content = ref(''), reason = ref(''), error = ref(''), loaded = ref(false)
async function load() { error.value = ''; try { const [p, r] = await Promise.all([privacy.privacyPolicy(), privacy.privacyRequests()]); policy.value = p; requests.value = r; loaded.value = true } catch (e) { error.value = e instanceof Error ? e.message : '查询失败'; throw e } }
const scope = usePrivateScope(() => { content.value = '' })
onUnmounted(onSessionExpired(scope.clear))
function clear() { scope.clear() }
async function viewCopy(id: string) { const stamp = scope.capture(); const result = await privacy.privacyCopy(id); if (scope.current(stamp)) content.value = JSON.stringify(result.content, null, 2) }
watch(() => props.revision, () => { void props.run(load) }, { immediate: true })
async function close() { const answer = await uni.showModal({ title: '申请注销核查', content: '未结任务、申请与争议将阻止注销。必要数字记录、原申请快照及证据受限保留，审批后账号将失效。' }); if (answer.confirm) await props.submit('privacy_close_request', { reason: reason.value }) }
</script>
<template><view class="card"><text class="heading">隐私、副本与注销核查</text><text v-if="error">{{ error }}；请重试，不代表没有资料。</text><text v-else-if="!loaded">正在加载隐私政策与本人申请…</text><text v-if="policy">{{ (policy.content as Record<string, unknown>).notice }}</text><text v-else-if="loaded">相关政策尚未发布，原业务记录保留。</text><button :disabled="busy" @click="props.run(load)">刷新隐私政策与申请</button><input v-model="reason" aria-label="本人副本或注销申请用途" placeholder="申请用途／原因" /><button :disabled="busy || blocked || !policy || !reason.trim()" @click="props.run(async () => { await submit('privacy_copy_request', { reason }) })">申请本人资料副本</button><button :disabled="busy || blocked || !policy || !reason.trim()" @click="props.run(close)">申请注销并核查未结事项</button><text v-if="loaded && !error && requests.length === 0">暂无本人资料治理申请。</text><view v-for="r in requests" :key="r.id"><text>{{ r.kind === 'copy' ? '本人资料副本' : '注销核查' }} · {{ r.id }} · {{ r.status }}</text><button v-if="r.kind === 'copy'" :disabled="busy" @click="props.run(() => viewCopy(r.id))">授权查看本人副本</button></view><view v-if="content"><text selectable>{{ content }}</text><button @click="clear">关闭并清除副本预览</button></view></view></template>

<style scoped src="../../shared/business-center.css"></style>
