<template>
  <div class="page-container">
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

    <div class="toolbar-card">
      <div class="toolbar-row">
        <n-input v-model:value="logStore.searchQuery" :placeholder="t('log.searchLogs')" clearable>
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
        <n-select
          v-model:value="logStore.filterType"
          clearable
          :options="logTypeOptions"
          :placeholder="t('log.filterType')"
        />
        <n-select
          v-model:value="logStore.groupingKey"
          clearable
          :options="groupingOptions"
          :placeholder="labels.grouping"
        />
        <n-select v-model:value="logStore.sortKey" :options="sortOptions" />
        <n-button quaternary @click="logStore.sortDesc = !logStore.sortDesc">
          <template #icon>
            <n-icon>
              <ArrowDownOutline v-if="logStore.sortDesc" />
              <ArrowUpOutline v-else />
            </n-icon>
          </template>
          {{ labels.sortOrder }}
        </n-button>
      </div>

      <div class="stats-row">
        <n-tag size="small" round :bordered="false">{{ t('log.records') }}: {{ logStore.logs.length }}</n-tag>
        <n-tag size="small" round :bordered="false" type="warning">{{ labels.filtered }}: {{ sortedLogs.length }}</n-tag>
        <n-tag size="small" round :bordered="false" type="info">{{ labels.status }}: {{ logStore.paused ? labels.paused : labels.streaming }}</n-tag>
      </div>
    </div>

    <div v-if="groupedLogs.length" class="logs-card">
      <template v-for="group in groupedLogs" :key="group.key">
        <div v-if="group.key" class="group-row">
          <span>{{ group.key }}</span>
          <n-tag size="tiny" round>{{ group.items.length }}</n-tag>
        </div>

        <div v-for="log in group.items" :key="log.seq" class="log-row" :class="log.type">
          <div class="log-meta">
            <span>#{{ log.seq }}</span>
            <span>{{ log.type.toUpperCase() }}</span>
            <span>{{ formatTime(log.timestamp) }}</span>
          </div>
          <div class="log-payload">{{ log.payload }}</div>
        </div>
      </template>
    </div>

    <div v-else class="empty-state">
      <div class="empty-icon">
        <n-icon size="48"><DocumentTextOutline /></n-icon>
      </div>
      <h3 class="empty-title">{{ t('log.noLogs') }}</h3>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMessage } from 'naive-ui'
import {
  ArrowDownOutline,
  ArrowUpOutline,
  DocumentTextOutline,
  SearchOutline,
} from '@vicons/ionicons5'
import PageHeader from '@/components/common/PageHeader.vue'
import { useLogStore } from '@/stores/kernel/LogStore'
import { useI18n } from 'vue-i18n'

defineOptions({
  name: 'LogView',
})

const { t, locale } = useI18n()
const message = useMessage()
const logStore = useLogStore()

const labels = computed(() => ({
  pause: locale.value.startsWith('zh') ? '暂停接收' : 'Pause',
  resume: locale.value.startsWith('zh') ? '恢复接收' : 'Resume',
  sortOrder: locale.value.startsWith('zh') ? '顺序' : 'Order',
  grouping: locale.value.startsWith('zh') ? '分组' : 'Grouping',
  filtered: locale.value.startsWith('zh') ? '筛选后' : 'Filtered',
  status: locale.value.startsWith('zh') ? '状态' : 'Status',
  paused: locale.value.startsWith('zh') ? '已暂停' : 'Paused',
  streaming: locale.value.startsWith('zh') ? '实时流' : 'Streaming',
}))

const logTypeOptions = computed(() => {
  const types = Array.from(new Set(logStore.logs.map((log) => log.type)))
  return types.map((type) => ({ label: type.toUpperCase(), value: type }))
})

const sortOptions = computed(() => [
  {
    label: locale.value.startsWith('zh') ? '序号' : 'Sequence',
    value: 'seq',
  },
  {
    label: locale.value.startsWith('zh') ? '级别' : 'Level',
    value: 'type',
  },
  {
    label: locale.value.startsWith('zh') ? '时间' : 'Time',
    value: 'timestamp',
  },
])

const groupingOptions = computed(() => [
  { label: locale.value.startsWith('zh') ? '按级别' : 'Level', value: 'type' },
  { label: locale.value.startsWith('zh') ? '按日期' : 'Date', value: 'date' },
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
  } catch (error) {
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

const formatTime = (timestamp: number) => new Date(timestamp).toLocaleString()
const formatDate = (timestamp: number) => new Date(timestamp).toLocaleDateString()
</script>

<style scoped>
.page-container {
  padding: var(--layout-page-padding-y, 16px) var(--layout-page-padding-x, 24px);
  max-width: var(--layout-page-max-width, 1400px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--layout-page-gap, 16px);
}

.toolbar-card,
.logs-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 16px;
}

.toolbar-row {
  display: grid;
  grid-template-columns: minmax(220px, 1fr) 180px 180px 180px auto;
  gap: 12px;
  align-items: center;
}

.stats-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.group-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 6px 0;
  font-weight: 600;
}

.log-row {
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 12px;
  margin-top: 10px;
  background: rgba(255, 255, 255, 0.02);
}

.log-row.error {
  border-color: rgba(239, 68, 68, 0.35);
}

.log-row.warning {
  border-color: rgba(245, 158, 11, 0.35);
}

.log-row.info {
  border-color: rgba(59, 130, 246, 0.35);
}

.log-meta {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
}

.log-payload {
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-primary);
  line-height: 1.5;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  color: var(--text-secondary);
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
}

@media (max-width: 960px) {
  .toolbar-row {
    grid-template-columns: 1fr;
  }
}
</style>
