<template>
  <div class="page-shell log-page">
    <PageHeader :title="t('log.title')" :subtitle="t('log.subtitle')">
      <template #actions>
        <n-space>
          <n-button secondary @click="logStore.togglePaused()">
            {{ logStore.paused ? labels.resume : labels.pause }}
          </n-button>
          <n-button secondary @click="copyLogs">{{ t('log.copy') }}</n-button>
          <n-button secondary @click="exportLogs">{{ t('log.export') }}</n-button>
          <n-button type="error" secondary @click="clearLogs">{{ t('log.clear') }}</n-button>
        </n-space>
      </template>
    </PageHeader>

    <ToolbarBar>
      <template #filters>
        <n-input v-model:value="logStore.searchQuery" :placeholder="t('log.searchLogs')" clearable size="small">
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
        <n-select
          v-model:value="logStore.filterType"
          clearable
          :options="logTypeOptions"
          :placeholder="t('log.filterType')"
          size="small"
        />
        <n-select
          v-model:value="logStore.groupingKey"
          clearable
          :options="groupingOptions"
          :placeholder="labels.grouping"
          size="small"
        />
        <n-select v-model:value="logStore.sortKey" :options="sortOptions" size="small" />
        <n-button size="small" quaternary @click="logStore.sortDesc = !logStore.sortDesc">
          <template #icon>
            <n-icon>
              <ArrowDownOutline v-if="logStore.sortDesc" />
              <ArrowUpOutline v-else />
            </n-icon>
          </template>
          {{ labels.sortOrder }}
        </n-button>
      </template>
      <template #stats>
        <n-tag size="small" round :bordered="false">{{ t('log.records') }}: {{ logStore.logs.length }}</n-tag>
        <n-tag size="small" round :bordered="false" type="warning">{{ labels.filtered }}: {{ sortedLogs.length }}</n-tag>
        <n-tag size="small" round :bordered="false" type="info">{{ labels.status }}: {{ logStore.paused ? labels.paused : labels.streaming }}</n-tag>
      </template>
    </ToolbarBar>

    <div v-if="groupedLogs.length" class="logs-card">
      <div class="log-table-wrap">
        <table class="log-table">
          <thead>
            <tr>
              <th>{{ t('log.sequence') }}</th>
              <th>{{ t('log.level') }}</th>
              <th>{{ t('log.time') }}</th>
              <th>{{ t('log.content') }}</th>
            </tr>
          </thead>
          <tbody v-for="group in groupedLogs" :key="group.key || 'all'">
            <tr v-if="group.key" class="group-row">
              <td colspan="4">
                <div class="group-title">
                  <span>{{ group.key }}</span>
                  <n-tag size="tiny" round>{{ group.items.length }}</n-tag>
                </div>
              </td>
            </tr>
            <tr
              v-for="log in group.items"
              :key="log.seq"
              class="log-row"
              :class="log.type"
              tabindex="0"
              @click="selectedLog = log"
              @keydown.enter="selectedLog = log"
              @keydown.space.prevent="selectedLog = log"
            >
              <td class="seq-cell">#{{ log.seq }}</td>
              <td>
                <n-tag size="small" round :bordered="false" :type="getLogTagType(log.type)">
                  {{ log.type.toUpperCase() }}
                </n-tag>
              </td>
              <td class="time-cell">{{ formatTime(log.timestamp) }}</td>
              <td class="payload-cell" :title="log.payload">{{ log.payload }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <EmptyState v-else :title="t('log.noLogs')" :icon="DocumentTextOutline" />

    <n-modal v-model:show="detailVisible" preset="card" :title="t('log.detailTitle')" style="width: 720px">
      <div v-if="selectedLog" class="detail-grid">
        <div><strong>{{ t('log.sequence') }}</strong><span>#{{ selectedLog.seq }}</span></div>
        <div><strong>{{ t('log.level') }}</strong><span>{{ selectedLog.type.toUpperCase() }}</span></div>
        <div><strong>{{ t('log.time') }}</strong><span>{{ formatTime(selectedLog.timestamp) }}</span></div>
        <div class="detail-payload">
          <strong>{{ t('log.content') }}</strong>
          <span>{{ selectedLog.payload }}</span>
        </div>
      </div>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useMessage } from 'naive-ui'
import {
  ArrowDownOutline,
  ArrowUpOutline,
  DocumentTextOutline,
  SearchOutline,
} from '@vicons/ionicons5'
import PageHeader from '@/components/common/PageHeader.vue'
import ToolbarBar from '@/components/common/ToolbarBar.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import { useLogStore, type LogEntry } from '@/stores/kernel/LogStore'
import { useI18n } from 'vue-i18n'

defineOptions({
  name: 'LogView',
})

const { t } = useI18n()
const message = useMessage()
const logStore = useLogStore()
const selectedLog = ref<LogEntry | null>(null)

const labels = computed(() => ({
  pause: t('log.pause'),
  resume: t('log.resume'),
  sortOrder: t('log.sortOrder'),
  grouping: t('log.grouping'),
  filtered: t('log.filtered'),
  status: t('log.status'),
  paused: t('log.paused'),
  streaming: t('log.streaming'),
}))

const detailVisible = computed({
  get: () => !!selectedLog.value,
  set: (value: boolean) => {
    if (!value) {
      selectedLog.value = null
    }
  },
})

const logTypeOptions = computed(() => {
  const types = Array.from(new Set(logStore.logs.map((log) => log.type)))
  return types.map((type) => ({ label: type.toUpperCase(), value: type }))
})

const sortOptions = computed(() => [
  {
    label: t('log.sequence'),
    value: 'seq',
  },
  {
    label: t('log.level'),
    value: 'type',
  },
  {
    label: t('log.time'),
    value: 'timestamp',
  },
])

const groupingOptions = computed(() => [
  { label: t('log.groupByLevel'), value: 'type' },
  { label: t('log.groupByDate'), value: 'date' },
])

const sortedLogs = computed(() => {
  const list = [...logStore.filteredLogs]
  const factor = logStore.sortDesc ? -1 : 1

  return list.sort((left, right) => {
    const leftValue = left[logStore.sortKey]
    const rightValue = right[logStore.sortKey]

    if (typeof leftValue === 'number' && typeof rightValue === 'number') {
      return (leftValue - rightValue) * factor
    }

    return String(leftValue).localeCompare(String(rightValue)) * factor
  })
})

const groupedLogs = computed(() => {
  const groupingKey = logStore.groupingKey
  if (!groupingKey) {
    return [{ key: '', items: sortedLogs.value }]
  }

  const groups = new Map<string, typeof sortedLogs.value>()
  sortedLogs.value.forEach((log) => {
    const key = groupingKey === 'type' ? log.type : formatDate(log.timestamp)
    const items = groups.get(key) || []
    items.push(log)
    groups.set(key, items)
  })

  return Array.from(groups.entries()).map(([key, items]) => ({ key, items }))
})

const clearLogs = () => {
  logStore.clearLogs()
  message.success(t('log.clearedSuccess'))
}

const copyLogs = async () => {
  try {
    await navigator.clipboard.writeText(sortedLogs.value.map((log) => `${formatTime(log.timestamp)} [${log.type}] ${log.payload}`).join('\n'))
    message.success(t('log.copiedSuccess'))
  } catch {
    message.error(t('log.copyFailed'))
  }
}

const exportLogs = () => {
  const content = sortedLogs.value
    .map((log) => `${formatTime(log.timestamp)} [${log.type}] ${log.payload}`)
    .join('\n')
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = `sing-box-windows-logs-${Date.now()}.txt`
  anchor.click()
  URL.revokeObjectURL(url)
  message.success(t('log.exportedSuccess'))
}

const getLogTagType = (type: string) => {
  const value = type.toLowerCase()
  if (value === 'error') return 'error'
  if (value === 'warning') return 'warning'
  if (value === 'success') return 'success'
  if (value === 'info') return 'info'
  return 'default'
}

const formatTime = (timestamp: number) => new Date(timestamp).toLocaleString()
const formatDate = (timestamp: number) => new Date(timestamp).toLocaleDateString()
</script>

<style scoped>
.log-page {
  max-width: var(--content-max-width, 1440px);
  margin: 0 auto;
}

.logs-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--panel-shadow);
  padding: var(--space-3) var(--space-4);
}

.log-table-wrap {
  overflow-x: auto;
}

.log-table {
  width: 100%;
  min-width: 860px;
  border-collapse: collapse;
  table-layout: fixed;
}

.log-table th {
  padding: 0 var(--space-3) var(--space-3);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: 600;
  text-align: left;
  white-space: nowrap;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.log-table td {
  padding: var(--space-3);
  border-top: 1px solid var(--border-color);
  color: var(--text-secondary);
  vertical-align: middle;
  font-size: var(--text-sm);
}

.log-table th:nth-child(1),
.log-table td:nth-child(1) {
  width: 86px;
}

.log-table th:nth-child(2),
.log-table td:nth-child(2) {
  width: 112px;
}

.log-table th:nth-child(3),
.log-table td:nth-child(3) {
  width: 190px;
}

.seq-cell,
.time-cell {
  font-family: var(--font-mono);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.group-row td {
  padding: var(--space-4) var(--space-3) var(--space-2);
  border-top: 0;
  background: transparent;
}

.group-title {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  font-weight: 600;
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.log-row {
  cursor: pointer;
  outline: none;
  transition: background-color var(--transition-fast);
}

.log-row:hover,
.log-row:focus-visible {
  background: var(--bg-surface-2);
}

.log-row.error .payload-cell {
  color: var(--error-color);
}

.log-row.warning .payload-cell {
  color: var(--warning-color);
}

.payload-cell {
  overflow: hidden;
  color: var(--text-primary);
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono);
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.detail-grid div {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.detail-grid strong {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-weight: 600;
}

.detail-grid span {
  word-break: break-word;
  font-size: var(--text-sm);
  color: var(--text-primary);
}

.detail-payload {
  grid-column: 1 / -1;
}

.detail-payload span {
  white-space: pre-wrap;
  line-height: 1.5;
  font-family: var(--font-mono);
}

@media (max-width: 960px) {
  .detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>
