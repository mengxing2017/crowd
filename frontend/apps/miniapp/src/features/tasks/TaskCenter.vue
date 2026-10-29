<script setup lang="ts">
import { tasks } from './api'
import { onUnmounted, ref, watch } from 'vue'
import { usePrivateFiles } from '../files/private-files'
import { chooseImagePath } from '../files/api'
import { onCommandRecovered } from '../platform/recovery'
import { fileUrl, upload } from '../files/api'
import { taskStatusNames, type Evidence, type TaskInstance, type TaskPage } from './api'
const props = defineProps<{ busy: boolean; pending: boolean; revision: number; submit: (action: string, body: Record<string, unknown>) => Promise<Record<string, unknown>>; run: (work: () => Promise<void>) => Promise<void> }>()
const page = ref<TaskPage>()
const selected = ref<TaskInstance>()
const status = ref('')
const draft = ref<Evidence>({ fields: {} })
const occurred = ref('')
const confirmed = ref(false)
const reason = ref('')
const previewImages = ref<string[]>([])
const privateFiles = usePrivateFiles(() => { previewImages.value = [] })
const stopRecovery = onCommandRecovered(({ action, result }) => { if (action.startsWith('evidence_') || action.startsWith('task_')) { if (typeof result.id === 'string' && typeof result.version === 'number') { selected.value = result as unknown as TaskInstance; resetDraft() } } })
onUnmounted(stopRecovery)
const statuses = Object.entries(taskStatusNames)
const loading = ref(false), loadError = ref('')
let loadRevision = 0
async function load(more = false) {
  const stamp = ++loadRevision
  loading.value = true; loadError.value = ''
  try {
  const next = await tasks.mine(JSON.stringify({ status: status.value || undefined, cursor: more ? page.value?.next_cursor : undefined }))
  if (stamp !== loadRevision) return
  page.value = more && page.value ? { ...next, items: [...page.value.items, ...next.items] } : next
  if (selected.value) { const id = selected.value.id; const detail = await tasks.detail(id); if (stamp === loadRevision && selected.value?.id === id) selected.value = detail }
  } catch (e) { if (stamp !== loadRevision) return; loadError.value = e instanceof Error ? e.message : '任务加载失败'; throw e }
  finally { if (stamp === loadRevision) loading.value = false }
}
watch(() => props.revision, () => { void props.run(() => load()) }, { immediate: true })
function resetDraft() { const t=selected.value; draft.value=JSON.parse(JSON.stringify(t?.draft || (t?.status==='supplement' ? t.receipt?.evidence : undefined) || { fields: {} })) as Evidence; occurred.value=draft.value.occurred_at ? new Date(draft.value.occurred_at*1000).toISOString() : ''; confirmed.value=false }
async function open(t: TaskInstance) { privateFiles.clear(); selected.value = await tasks.detail(t.id); resetDraft(); reason.value = ''; previewImages.value = [] }
async function act(action: string, body: Record<string, unknown> = {}) {
  if (!selected.value) return
  const result = await props.submit(action, { id: selected.value.id, version: selected.value.version, ...body })
  if (result.id && typeof result.version === 'number') selected.value = result as unknown as TaskInstance
  await load().catch(() => undefined)
}
async function choose(field: string) {
  if (!selected.value) return
  if (!editable(field)) throw new Error('此字段不在本次补交范围内')
  const path = await chooseImagePath(true)
  let file
  try { file = await upload(path, `task_evidence:${selected.value.id}`) }
  catch (e) { throw new Error(`“${selected.value.template.fields.find(f => f.id === field)?.label || field}”上传失败：${e instanceof Error ? e.message : '请重试'}；其他已上传证据已保留`) }
  if (!editable(field)) throw new Error('补交范围已变化，请重新查看任务')
  const current = draft.value.fields[field]
  draft.value.fields[field] = [...(Array.isArray(current) ? current : []), file.id]
  invalidatePreview()
}
async function saveDraft() {
  const time = occurred.value ? Math.floor(new Date(occurred.value).getTime() / 1000) : undefined
  if (occurred.value && !Number.isSafeInteger(time)) throw new Error('请填写可识别的成果发生时间，如 2026-10-28T12:30:00+08:00')
  await act('evidence_save', { evidence: { fields: draft.value.fields, ...(time !== undefined ? { occurred_at: time } : {}) } })
}
async function preview() {
  if (!selected.value) return
  for (const field of selected.value.template.fields) {
    const value = draft.value.fields[field.id]
    if (field.required && (!value || (Array.isArray(value) ? !value.length : !value.trim()))) throw new Error(`请补齐“${field.label}”`)
    if (field.kind === 'text' && typeof value === 'string' && [...value].length > (field.max_length || 0)) throw new Error(`“${field.label}”超过最大字数`)
  }
  await saveDraft(); await act('evidence_preview'); confirmed.value = false }
function invalidatePreview() { confirmed.value = false; if (selected.value) selected.value.preview = undefined }
function editable(id: string) { return selected.value?.status !== 'supplement' || !!selected.value.supplement_fields?.includes(id) }
function removeImage(field: string, id: string) { if (!editable(field)) return; draft.value.fields[field] = (draft.value.fields[field] as string[]).filter(item => item !== id); invalidatePreview() }
function setField(id: string, event: unknown) { if (!editable(id)) return; draft.value.fields[id] = (event as { detail: { value: string } }).detail.value; invalidatePreview() }
async function showImage(id: string) { privateFiles.clear(); const path = await privateFiles.download(fileUrl(id, '本人核对任务证据')); if (path) previewImages.value = [path] }
</script>
<template>
  <view class="card">
    <text class="heading">我的任务执行与提交</text><text>佣金为标记数字；审核通过后等待数字入账，不代表自动到账。</text>
    <picker :range="['全部状态', ...statuses.map(([, name]) => name)]" @change="status = statuses[Number($event.detail.value) - 1]?.[0] || ''; props.run(() => load())"><view>筛选：{{ taskStatusNames[status] || '全部状态' }}</view></picker>
    <button :disabled="busy || pending" @click="props.run(() => load())">刷新我的任务与回执</button>
    <text v-if="loading">{{ page ? '正在刷新任务，保留上次查询结果…' : '正在首次加载任务…' }}</text>
    <text v-if="loadError">{{ loadError }}；请重试，不代表没有任务。</text>
    <text v-if="!loading && !loadError && page?.items.length === 0">暂无匹配任务；旧版领取保留在历史快照中。</text>
    <view v-for="t in page?.items || []" :key="t.id" class="record"><text>{{ t.product.name }} · {{ t.template.name }}</text><text>{{ taskStatusNames[t.status] || t.status }} · {{ t.id }}</text><button :disabled="busy || pending" @click="props.run(() => open(t))">查看任务与下一步</button></view>
    <button v-if="page?.next_cursor" :disabled="busy || pending" @click="props.run(() => load(true))">加载更多任务</button>
    <view v-if="selected" class="record">
      <text class="heading">{{ selected.template.name }} · {{ taskStatusNames[selected.status] }}</text><text>任务编号：{{ selected.id }}；版本：{{ selected.version }}</text>
      <text>完成条件：{{ selected.template.completion }}</text><text>失败条件：{{ selected.template.failure_conditions }}</text><text>审核目标：{{ selected.template.review_goal }}</text>
      <text>执行截止：{{ new Date(selected.execute_until * 1000).toLocaleString() }}；提交截止：{{ new Date(selected.submit_until * 1000).toLocaleString() }}</text>
      <text v-if="selected.price.kind === 'fixed'">固定佣金标记 {{ selected.reserved_cents }} 分</text><text v-else>比例 {{ selected.price.numerator }}/{{ selected.price.denominator }}；舍入 {{ selected.price.rounding }}；{{ selected.price.cap_cents == null ? '未配置单笔封顶，数字待核定。' : `单笔封顶 ${selected.price.cap_cents} 分。` }}实际数字以可信成果基数计算。</text>
      <button v-if="selected.status === 'claimed'" :disabled="busy || pending" @click="props.run(() => act('task_start'))">开始执行（记录首次开始时间）</button>
      <view v-for="step in selected.template.steps" :key="step.id"><text>{{ selected.steps.includes(step.id) ? '已完成' : '未完成' }} · {{ step.title }}（{{ step.required ? '必做' : '选做' }}）</text><button v-if="['in_progress', 'supplement'].includes(selected.status) && !selected.steps.includes(step.id)" :disabled="busy || pending" @click="props.run(() => act('task_step', { step_id: step.id }))">确认完成此步骤</button></view>
      <view v-if="['in_progress', 'supplement'].includes(selected.status)">
        <text v-if="selected.status === 'supplement'">需补交：{{ selected.supplement_fields?.join('、') }}；截止 {{ new Date((selected.supplement_until || 0) * 1000).toLocaleString() }}。其余字段保持原版本。</text>
        <view v-for="field in selected.template.fields" :key="field.id"><text>{{ field.label }}（{{ field.required ? '必填' : '选填' }}）：{{ field.purpose }}</text>
          <template v-if="field.kind === 'image'"><view v-for="id in (Array.isArray(draft.fields[field.id]) ? draft.fields[field.id] : [])" :key="id"><button :disabled="busy || pending" @click="props.run(() => showImage(String(id)))">查看已选私有证据图片</button><button :disabled="busy || pending || !editable(field.id)" @click="removeImage(field.id, String(id))">从本次草稿移除</button></view><button :disabled="busy || pending || !editable(field.id)" @click="props.run(() => choose(field.id))">添加证据图片</button></template>
          <textarea v-else :maxlength="-1" :value="draft.fields[field.id] as string" :aria-label="field.kind === 'order' ? '来源订单号（字母、数字、横线、下划线）' : field.label" :placeholder="field.kind === 'order' ? '来源订单号（字母、数字、横线、下划线）' : field.label" :disabled="busy || pending || (selected.status === 'supplement' && !selected.supplement_fields?.includes(field.id))" @input="setField(field.id, $event)" />
        </view>
        <input v-model="occurred" :disabled="selected.status === 'supplement'" aria-label="成果发生时间，例如 2026-10-28T12:30:00+08:00" placeholder="成果发生时间，例如 2026-10-28T12:30:00+08:00" @input="invalidatePreview" />
        <button :disabled="busy || pending" @click="props.run(async () => { await act('evidence_discard'); resetDraft() })">清理未提交草稿与未引用附件</button><button :disabled="busy || pending" @click="props.run(saveDraft)">保存证据草稿（不进入审核）</button><button :disabled="busy || pending" @click="props.run(preview)">保存并预览本次提交</button>
        <view v-if="selected.preview"><text>最终预览编号：{{ selected.preview.id }}</text><text v-for="(value, key) in selected.preview.evidence.fields" :key="key">{{ selected.template.fields.find(f => f.id === key)?.label }}：{{ value }}</text><text>发生时间：{{ selected.preview.evidence.occurred_at ? new Date(selected.preview.evidence.occurred_at * 1000).toLocaleString() : '' }}</text><checkbox-group @change="confirmed = $event.detail.value.includes('evidence')"><label><checkbox value="evidence" :checked="confirmed" />我确认以上预览证据真实、完整，并提交这一固定版本</label></checkbox-group><button :disabled="busy || pending || !confirmed" @click="props.run(() => act('evidence_submit', { preview_id: selected?.preview?.id, confirmed }))">确认提交并获取回执</button></view>
      </view>
      <image v-for="path in previewImages" :key="path" :src="path" mode="widthFix" />
      <view v-if="selected.receipt"><text>提交回执：{{ selected.receipt.id }}；证据第 {{ selected.receipt.version }} 版；{{ new Date(selected.receipt.submitted_at * 1000).toLocaleString() }}；{{ taskStatusNames[selected.status] }}</text></view>
      <text v-for="(d, i) in selected.decisions" :key="i">审核说明：{{ d.reason }}</text>
      <input v-model="reason" aria-label="放弃／撤回原因" placeholder="放弃／撤回原因" />
      <button v-if="['claimed', 'in_progress'].includes(selected.status) && selected.template.allow_abandon" :disabled="busy || pending || !reason" @click="props.run(() => act('task_abandon', { reason }))">放弃任务并保留记录</button>
      <button v-if="selected.status === 'submitted' && selected.template.allow_withdraw" :disabled="busy || pending || !reason" @click="props.run(() => act('evidence_withdraw', { reason }))">审核领取前撤回（保留提交历史）</button>
      <text v-if="selected.status === 'paused'">平台已暂停，请等待恢复；期限按领取时暂停策略处理。</text>
      <button @click="selected = undefined; privateFiles.clear()">收起任务详情</button>
    </view>
  </view>
</template>

<style scoped src="../../shared/business-center.css"></style>
