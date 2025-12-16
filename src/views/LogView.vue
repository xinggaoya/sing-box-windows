<template>
  <div class="page-container">
    <PageHeader :title="t('log.title')" :subtitle="t('log.subtitle')">
      <template #actions>
        <div class="header-controls">
          <!-- 简化的统计信息 -->
          <div class="mini-stats">
            <n-tag type="info" size="small">
              {{ totalLogs }} {{ t('log.records') }}
            </n-tag>
            <n-tag v-if="logTypeCounts.error" type="error" size="small">
              {{ t('log.types.error') }}: {{ logTypeCounts.error }}
            </n-tag>
            <n-tag v-if="logTypeCounts.warning" type="warning" size="small">
              {{ t('log.types.warning') }}: {{ logTypeCounts.warning }}
            </n-tag>
          </div>

          <n-switch v-model:value="autoScroll" size="medium">
            <template #checked>{{ t('log.autoScroll') }}</template>
            <template #unchecked>{{ t('log.manualScroll') }}</template>
          </n-switch>

          <n-button-group>
            <n-button
              @click="clearLogs"
              :disabled="!displayedLogs.length"
              size="medium"
              secondary
              type="error"
            >
              <template #icon>
                <n-icon><TrashOutline /></n-icon>
              </template>
            </n-button>
            <n-button
              @click="copyLogs"
              :disabled="!displayedLogs.length"
              size="medium"
              secondary
            >
              <template #icon>
                <n-icon><CopyOutline /></n-icon>
              </template>
            </n-button>
            <n-button
              @click="exportLogs"
              :disabled="!displayedLogs.length"
              size="medium"
              secondary
            >
              <template #icon>
                <n-icon><DownloadOutline /></n-icon>
              </template>
            </n-button>
          </n-button-group>
        </div>
      </template>
    </PageHeader>

    <!-- Filters -->
    <div class="filter-section">
      <div class="filter-bar">
        <n-input
          v-model:value="searchQuery"
          :placeholder="t('log.searchLogs')"
          clearable
          round
          class="search-input"
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
        
        <n-select
          v-model:value="filterType"
          :options="logTypeOptions"
          :placeholder="t('log.filterType')"
          clearable
          class="filter-select"
        />
      </div>
    </div>

    <!-- Logs List -->
    <div class="logs-section">
      <div class="logs-container" ref="logListRef" @scroll="handleScroll">
        <!-- 快速定位按钮 -->
        <div v-if="displayedLogs.length > 0" class="scroll-controls">
          <n-button-group size="small" vertical>
            <n-button
              @click="scrollToTop"
              :disabled="!isScrolled"
              circle
              type="tertiary"
            >
              <template #icon>
                <n-icon><ChevronUpOutline /></n-icon>
              </template>
            </n-button>
            <n-button
              @click="scrollToBottom"
              :disabled="isAtBottom"
              circle
              type="tertiary"
            >
              <template #icon>
                <n-icon><ChevronDownOutline /></n-icon>
              </template>
            </n-button>
          </n-button-group>
        </div>
        <div v-if="displayedLogs.length > 0" class="logs-list">
          <div
            v-for="log in displayedLogs"
            :key="log.key"
            class="log-entry"
            :class="log.type"
          >
            <div class="log-meta">
              <span class="log-time">{{ formatLogTime(log.timestamp) }}</span>
              <span class="log-type-badge">{{ getLogTypeLabel(log.type) }}</span>
            </div>
            <div class="log-content">
              <HighlightText
                v-if="searchQuery"
                :text="log.payload"
                :keyword="searchQuery"
              />
              <span v-else>{{ log.payload }}</span>
            </div>
          </div>
        </div>

        <!-- Empty State -->
        <div v-else class="empty-state">
          <div class="empty-icon">
            <n-icon size="48"><DocumentTextOutline /></n-icon>
          </div>
          <h3 class="empty-title">
            {{ searchQuery ? t('log.noSearchResults') : t('log.noLogs') }}
          </h3>
          <n-button
            v-if="searchQuery"
            @click="searchQuery = ''"
            secondary
          >
            {{ t('log.clearSearch') }}
          </n-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useLogStore } from '@/stores/kernel/LogStore'
import { ref, computed, watch, nextTick, defineComponent, h } from 'vue'
import { useMessage } from 'naive-ui'
import {
  TrashOutline,
  CopyOutline,
  DownloadOutline,
  DocumentTextOutline,
  SearchOutline,
  InformationCircleOutline,
  WarningOutline,
  AlertCircleOutline,
  CheckmarkCircleOutline,
  ChevronUpOutline,
  ChevronDownOutline,
} from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import PageHeader from '@/components/common/PageHeader.vue'
import StatusCard from '@/components/common/StatusCard.vue'

defineOptions({
  name: 'LogView'
})

interface Log {
  type: string
  payload: string
  timestamp: number
}

interface FormattedLog extends Log {
  key: string
}

// Highlight Component
const HighlightText = defineComponent({
  name: 'HighlightText',
  props: {
    text: { type: String, required: true },
    keyword: { type: String, required: true },
  },
  setup(props) {
    return () => {
      if (!props.keyword) return props.text
      const parts = props.text.split(new RegExp(`(${props.keyword})`, 'gi'))
      return h('span', {}, parts.map((part, index) => {
        if (part.toLowerCase() === props.keyword.toLowerCase()) {
          return h('mark', {
            key: index,
            style: {
              backgroundColor: 'rgba(255, 255, 0, 0.2)',
              color: 'inherit',
              padding: '0 2px',
              borderRadius: '2px',
            }
          }, part)
        }
        return h('span', { key: index }, part)
      }))
    }
  }
})

const logStore = useLogStore()
const { t } = useI18n()
const message = useMessage()

const searchQuery = ref('')
const filterType = ref<string | null>(null)
const autoScroll = ref(false)
const logListRef = ref<HTMLElement | null>(null)
const isScrolled = ref(false)
const isAtBottom = ref(true)

// Computed
const logs = computed(() => logStore.logs)
const totalLogs = computed(() => logs.value.length)

const formattedLogs = computed<FormattedLog[]>(() => {
  return logs.value.map((log, index) => ({
    ...log,
    key: `${log.timestamp}-${index}`,
  }))
})

const displayedLogs = computed<FormattedLog[]>(() => {
  let filteredLogs = formattedLogs.value
  if (filterType.value) {
    filteredLogs = filteredLogs.filter((log) => log.type === filterType.value)
  }
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filteredLogs = filteredLogs.filter((log) =>
      log.payload.toLowerCase().includes(query)
    )
  }
  return filteredLogs
})

const logTypeCounts = computed(() => {
  const counts: Record<string, number> = {}
  displayedLogs.value.forEach((log) => {
    counts[log.type] = (counts[log.type] || 0) + 1
  })
  return counts
})

const logStats = computed(() => {
  const typeMap = [
    { type: 'info', icon: InformationCircleOutline, accent: 'primary' },
    { type: 'warning', icon: WarningOutline, accent: 'warning' },
    { type: 'error', icon: AlertCircleOutline, accent: 'error' },
    { type: 'success', icon: CheckmarkCircleOutline, accent: 'success' },
  ]

  const stats = typeMap.map((config) => ({
    label: getLogTypeLabel(config.type),
    value: logTypeCounts.value[config.type] || 0,
    icon: config.icon,
    type: config.accent as any,
  }))

  stats.push({
    label: t('log.records'),
    value: totalLogs.value,
    icon: DocumentTextOutline,
    type: 'default',
  })

  return stats
})

const logTypeOptions = computed(() => {
  const types = new Set<string>()
  logs.value.forEach((log) => {
    types.add(log.type)
  })
  return Array.from(types).map((type) => ({
    label: getLogTypeLabel(type),
    value: type,
  }))
})

// Methods
const getLogTypeLabel = (type: string): string => {
  const typeMap: Record<string, string> = {
    info: t('log.types.info'),
    warning: t('log.types.warning'),
    error: t('log.types.error'),
    success: t('log.types.success'),
  }
  return typeMap[type] || type.toUpperCase()
}

const formatLogTime = (timestamp: number): string => {
  return new Date(timestamp).toLocaleTimeString()
}

const clearLogs = () => {
  logStore.clearLogs()
  message.success(t('log.clearedSuccess'))
}

const copyLogs = () => {
  const logText = displayedLogs.value
    .map((log) => `[${formatLogTime(log.timestamp)}] [${log.type.toUpperCase()}] ${log.payload}`)
    .join('\n')

  navigator.clipboard.writeText(logText).then(() => {
    message.success(t('log.copiedSuccess'))
  }).catch(() => {
    message.error(t('log.copyFailed'))
  })
}

const exportLogs = () => {
  const logText = displayedLogs.value
    .map((log) => `[${new Date(log.timestamp).toISOString()}] [${log.type.toUpperCase()}] ${log.payload}`)
    .join('\n')

  const blob = new Blob([logText], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `logs-${new Date().toISOString().split('T')[0]}.txt`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  message.success(t('log.exportedSuccess'))
}

// 滚动控制方法
const scrollToTop = () => {
  if (logListRef.value) {
    logListRef.value.scrollTop = 0
  }
}

const scrollToBottom = () => {
  if (logListRef.value) {
    logListRef.value.scrollTop = logListRef.value.scrollHeight
  }
}

// 监听滚动事件
const handleScroll = () => {
  if (logListRef.value) {
    const { scrollTop, scrollHeight, clientHeight } = logListRef.value
    isScrolled.value = scrollTop > 0
    isAtBottom.value = scrollTop + clientHeight >= scrollHeight - 10
  }
}

watch(
  () => displayedLogs.value.length,
  async (newLength, oldLength) => {
    if (autoScroll.value) {
      await nextTick()
      if (logListRef.value) {
        // 自动滚动到底部（最旧的日志）
        logListRef.value.scrollTop = logListRef.value.scrollHeight
      }
    } else if (!oldLength && newLength > 0) {
      // 首次加载日志时，滚动到顶部（最新的日志）
      await nextTick()
      if (logListRef.value) {
        logListRef.value.scrollTop = 0
      }
    }
  }
)
</script>

<style scoped>
.page-container {
  padding: var(--layout-page-padding-y, 16px) var(--layout-page-padding-x, 24px);
  max-width: var(--layout-page-max-width, 1400px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--layout-page-gap, 16px);
  height: calc(100vh - 40px); /* Adjust for window controls */
}

.header-controls {
  display: flex;
  align-items: center;
  gap: var(--layout-row-gap, 12px);
  flex-wrap: wrap;
}

.mini-stats {
  display: flex;
  gap: 8px;
  align-items: center;
}

.filter-section {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 12px 16px;
  flex-shrink: 0;
}

.filter-bar {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.search-input {
  flex: 2;
  min-width: 200px;
}

.filter-select {
  flex: 1;
  min-width: 160px;
}

.logs-section {
  flex: 1;
  min-height: 0;
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.logs-container {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
  scroll-behavior: smooth;
  height: 100%;
  position: relative;
}

.scroll-controls {
  position: fixed;
  bottom: 24px;
  right: 32px;
  z-index: 100;
}

.logs-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.log-entry {
  display: flex;
  gap: 10px;
  padding: 6px 10px;
  border-radius: 6px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  line-height: 1.4;
  transition: background 0.1s ease;
}

.log-entry:hover {
  background: var(--bg-tertiary);
}

.log-meta {
  display: flex;
  align-items: baseline;
  gap: 6px;
  flex-shrink: 0;
  width: 120px;
}

.log-time {
  color: var(--text-tertiary);
  font-size: 11px;
}

.log-type-badge {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 1px 5px;
  border-radius: 3px;
}

.log-entry.info .log-type-badge {
  color: #3b82f6;
  background: rgba(59, 130, 246, 0.1);
}

.log-entry.warning .log-type-badge {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
}

.log-entry.error .log-type-badge {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

.log-entry.success .log-type-badge {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.log-content {
  color: var(--text-secondary);
  word-break: break-all;
}

.log-entry.error .log-content {
  color: #ef4444;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  color: var(--text-secondary);
  height: 100%;
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 16px;
  color: var(--text-primary);
}
</style>
