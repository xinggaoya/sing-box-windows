<template>
  <div class="page-shell logs-page" :style="pageThemeStyle">
    <section class="page-hero">
      <div class="hero-row">
        <div class="hero-left">
          <div class="hero-icon">
            <n-icon size="26">
              <DocumentTextOutline />
            </n-icon>
          </div>
          <div class="hero-meta">
            <p class="hero-subtitle">{{ t('log.subtitle') }}</p>
            <h2 class="hero-title">{{ t('log.title') }}</h2>
          </div>
        </div>
        <div class="hero-actions">
          <n-switch
            v-model:value="autoScroll"
            size="large"
            class="auto-scroll-switch"
          >
            <template #checked>{{ t('log.autoScroll') }}</template>
            <template #unchecked>{{ t('log.manualScroll') }}</template>
          </n-switch>
        </div>
      </div>
      <div class="hero-stats">
        <div
          v-for="stat in logStats"
          :key="stat.label"
          class="stat-card"
          :data-accent="stat.accent"
        >
          <div class="stat-icon">
            <n-icon :size="20">
              <component :is="stat.icon" />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stat.value }}</div>
            <div class="stat-label">{{ stat.label }}</div>
          </div>
        </div>
      </div>
    </section>

    <section class="page-section">
      <n-card class="surface-card filter-panel" :bordered="false">
        <div class="filter-content">
          <div class="filter-row">
            <n-input
              v-model:value="searchQuery"
              :placeholder="t('log.searchLogs')"
              clearable
              size="large"
              class="search-input"
            >
              <template #prefix>
                <n-icon size="16">
                  <SearchOutline />
                </n-icon>
              </template>
            </n-input>
            <n-select
              v-model:value="filterType"
              :options="logTypeOptions"
              :placeholder="t('log.filterType')"
              clearable
              size="large"
              class="filter-select"
            />
          </div>
          <div class="filter-actions">
            <n-button-group>
              <n-button
                @click="clearLogs"
                :disabled="!displayedLogs.length"
                type="error"
                size="medium"
              >
                <template #icon>
                  <n-icon size="16">
                    <TrashOutline />
                  </n-icon>
                </template>
                {{ t('log.clear') }}
              </n-button>
              <n-button
                @click="copyLogs"
                :disabled="!displayedLogs.length"
                type="info"
                size="medium"
              >
                <template #icon>
                  <n-icon size="16">
                    <CopyOutline />
                  </n-icon>
                </template>
                {{ t('log.copy') }}
              </n-button>
              <n-button
                @click="exportLogs"
                :disabled="!displayedLogs.length"
                type="success"
                size="medium"
              >
                <template #icon>
                  <n-icon size="16">
                    <DownloadOutline />
                  </n-icon>
                </template>
                {{ t('log.export') }}
              </n-button>
            </n-button-group>
          </div>
        </div>
      </n-card>

      <n-card class="surface-card logs-card" :bordered="false">
        <div class="logs-content">
          <div class="logs-header">
            <div class="logs-info">
              <span class="logs-count">
                {{ displayedLogs.length }}/{{ totalLogs }} {{ t('log.records') }}
              </span>
              <span class="logs-time" v-if="displayedLogs.length > 0">
                {{ getLatestLogTime() }}
              </span>
            </div>
          </div>

          <div v-if="displayedLogs.length > 0" class="logs-list">
            <div
              v-for="(log, index) in displayedLogs"
              :key="log.key"
              class="log-item"
              :class="`log-${log.type}`"
            >
              <div class="log-header">
                <div class="log-time">
                  {{ formatLogTime(log.timestamp) }}
                </div>
                <div class="log-type">
                  <n-tag :type="getLogTagType(log.type)" size="small">
                    {{ getLogTypeLabel(log.type) }}
                  </n-tag>
                </div>
              </div>
              <div class="log-content">
                <HighlightText
                  v-if="searchQuery"
                  :text="log.payload"
                  :keyword="searchQuery"
                />
                <span v-else class="log-text">{{ log.payload }}</span>
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-else class="empty-state">
            <div class="empty-icon">
              <n-icon size="48">
                <DocumentTextOutline />
              </n-icon>
            </div>
            <div class="empty-title">
              {{ searchQuery ? t('log.noSearchResults') : t('log.noLogs') }}
            </div>
            <div class="empty-desc">
              {{ searchQuery ? t('log.adjustSearchFilters') : t('log.noLogsDesc') }}
            </div>
            <n-button
              v-if="searchQuery"
              @click="searchQuery = ''"
              type="primary"
              size="large"
              class="empty-btn"
            >
              {{ t('log.clearSearch') }}
            </n-button>
          </div>
        </div>
      </n-card>
    </section>
  </div>
</template>

<script setup lang="ts">
import { useLogStore } from '@/stores/kernel/LogStore'
import { onMounted, ref, computed, onUnmounted, watch, nextTick, defineComponent, h } from 'vue'
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
} from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { usePageTheme } from '@/composables/usePageTheme'

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

// 高亮文本组件
const HighlightText = defineComponent({
  name: 'HighlightText',
  props: {
    text: {
      type: String,
      required: true,
    },
    keyword: {
      type: String,
      required: true,
    },
  },
  setup(props) {
    return () => {
      if (!props.keyword) {
        return props.text
      }

      const parts = props.text.split(new RegExp(`(${props.keyword})`, 'gi'))

      return h('span', {}, parts.map((part, index) => {
        if (part.toLowerCase() === props.keyword.toLowerCase()) {
          return h('mark', {
            key: index,
            style: {
              backgroundColor: 'rgba(91, 76, 253, 0.2)',
              color: '#5b4cfd',
              padding: '2px 4px',
              borderRadius: '4px',
              fontWeight: '600'
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
const themeStore = useThemeStore()
const pageThemeStyle = usePageTheme(themeStore)

// 响应式数据
const searchQuery = ref('')
const filterType = ref<string | null>(null)
const autoScroll = ref(true)
const logListRef = ref()

// 计算属性
const logs = computed(() => logStore.logs)
const totalLogs = computed(() => logs.value.length)

// 格式化日志数据
const formattedLogs = computed<FormattedLog[]>(() => {
  return logs.value.map((log, index) => ({
    ...log,
    key: `${log.timestamp}-${index}`,
  }))
})

// 筛选后的日志
const displayedLogs = computed<FormattedLog[]>(() => {
  let filteredLogs = formattedLogs.value

  // 按类型筛选
  if (filterType.value) {
    filteredLogs = filteredLogs.filter((log) => log.type === filterType.value)
  }

  // 按关键词搜索
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filteredLogs = filteredLogs.filter((log) =>
      log.payload.toLowerCase().includes(query)
    )
  }

  return filteredLogs
})

// 日志类型统计
const logTypeCounts = computed(() => {
  const counts: Record<string, number> = {}
  displayedLogs.value.forEach((log) => {
    counts[log.type] = (counts[log.type] || 0) + 1
  })
  return counts
})

const logStats = computed(() => {
  const typeMap = [
    { type: 'info', icon: InformationCircleOutline, accent: 'blue' },
    { type: 'warning', icon: WarningOutline, accent: 'amber' },
    { type: 'error', icon: AlertCircleOutline, accent: 'pink' },
    { type: 'success', icon: CheckmarkCircleOutline, accent: 'purple' },
  ]

  const stats = typeMap.map((config) => ({
    label: getLogTypeLabel(config.type),
    value: logTypeCounts.value[config.type] || 0,
    icon: config.icon,
    accent: config.accent,
  }))

  stats.push({
    label: t('log.records'),
    value: totalLogs.value,
    icon: DocumentTextOutline,
    accent: 'blue',
  })

  return stats
})

// 日志类型选项
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

// 辅助方法
const getLogTypeLabel = (type: string): string => {
  const typeMap: Record<string, string> = {
    info: t('log.types.info'),
    warning: t('log.types.warning'),
    error: t('log.types.error'),
    success: t('log.types.success'),
  }
  return typeMap[type] || type.toUpperCase()
}

const getLogTagType = (type: string): 'info' | 'warning' | 'error' | 'success' => {
  const typeMap: Record<string, 'info' | 'warning' | 'error' | 'success'> = {
    info: 'info',
    warning: 'warning',
    error: 'error',
    success: 'success',
  }
  return typeMap[type] || 'info'
}

const formatLogTime = (timestamp: number): string => {
  return new Date(timestamp).toLocaleTimeString()
}

const getLatestLogTime = (): string => {
  if (displayedLogs.value.length === 0) return ''
  const latestLog = displayedLogs.value[0]
  return t('log.latestAt', { time: formatLogTime(latestLog.timestamp) })
}

// 操作方法
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

// 监听新日志，自动滚动到底部
watch(
  () => displayedLogs.value.length,
  async () => {
    if (autoScroll.value) {
      await nextTick()
      if (logListRef.value) {
        logListRef.value.scrollTo({ top: 999999, behavior: 'smooth' })
      }
    }
  }
)

// 生命周期
onMounted(() => {
  logStore.setupLogListener()
})

onUnmounted(() => {
  logStore.cleanupListeners()
})
</script>

<style scoped>
.logs-page {
  animation: fadeIn 0.4s ease both;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.filter-panel {
  border-radius: 28px;
}

.filter-content {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.filter-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1;
}

.filter-select {
  min-width: 220px;
  flex: 1;
}

.filter-actions {
  display: flex;
  justify-content: flex-end;
}

.logs-card {
  border-radius: 32px;
}

.logs-content {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.logs-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--divider-color);
  padding-bottom: 12px;
  color: var(--text-muted);
}

.logs-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 540px;
  overflow-y: auto;
}

.log-item {
  border-radius: 20px;
  padding: 16px 20px;
  background: rgba(15, 23, 42, 0.02);
  border: 1px solid var(--panel-border);
  transition: border-color 0.2s ease, transform 0.2s ease;
}

.log-item:hover {
  border-color: rgba(91, 76, 253, 0.35);
  transform: translateY(-2px);
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  gap: 12px;
}

.log-time {
  font-size: 13px;
  color: var(--text-muted);
}

.log-content {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.6;
  word-break: break-word;
}

.log-item mark {
  background: rgba(91, 76, 253, 0.2);
  padding: 2px 4px;
  border-radius: 4px;
}

.logs-card .empty-state {
  margin-top: 12px;
}

@media (max-width: 768px) {
  .filter-select {
    min-width: 160px;
  }
}
</style>
