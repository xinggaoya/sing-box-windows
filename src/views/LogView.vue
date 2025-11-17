<template>
  <div class="logs-page">
    <!-- 页面标题和统计 -->
    <div class="page-header">
      <div class="header-content">
        <div class="header-left">
          <div class="header-icon">
            <n-icon size="20">
              <DocumentTextOutline />
            </n-icon>
          </div>
          <div class="header-info">
            <h1 class="page-title">{{ t('log.title') }}</h1>
            <p class="page-subtitle">{{ t('log.subtitle') }}</p>
          </div>
        </div>
        <div class="header-actions">
          <n-switch
            v-model:value="autoScroll"
            size="medium"
            class="auto-scroll-switch"
          >
            <template #checked>{{ t('log.autoScroll') }}</template>
            <template #unchecked>{{ t('log.manualScroll') }}</template>
          </n-switch>
        </div>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <n-card
        v-for="(count, type) in logTypeCounts"
        :key="type"
        class="stat-card"
        :class="`${type}-card`"
        :bordered="false"
      >
        <div class="stat-content">
          <div class="stat-icon">
            <n-icon size="24">
              <InformationCircleOutline v-if="type === 'info'" />
              <WarningOutline v-else-if="type === 'warning'" />
              <AlertCircleOutline v-else-if="type === 'error'" />
              <CheckmarkCircleOutline v-else-if="type === 'success'" />
              <DocumentTextOutline v-else />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ count }}</div>
            <div class="stat-label">{{ getLogTypeLabel(type) }}</div>
          </div>
        </div>
      </n-card>
    </div>

    <!-- 搜索和筛选 -->
    <n-card class="filter-card" :bordered="false">
      <div class="filter-content">
        <div class="filter-row">
          <n-input
            v-model:value="searchQuery"
            :placeholder="t('log.searchLogs')"
            clearable
            size="medium"
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
            size="medium"
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

    <!-- 日志列表 -->
    <n-card class="logs-card" :bordered="false">
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
  padding: 16px;
  min-height: calc(100vh - 48px);
  background: v-bind('themeStore.isDark ? "#18181b" : "#f8fafc"');
}

/* 页面标题 */
.page-header {
  margin-bottom: 24px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border-radius: 16px;
  padding: 24px 28px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0;
  letter-spacing: -0.02em;
}

.page-subtitle {
  font-size: 14px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0;
  font-weight: 400;
}

.header-actions {
  display: flex;
  gap: 16px;
  align-items: center;
}

.auto-scroll-switch {
  font-weight: 500;
}

/* 统计卡片 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 24px;
}

.stat-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
  transition: all 0.3s ease;
  overflow: hidden;
  position: relative;
}

.stat-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
}

.info-card::before {
  background: linear-gradient(90deg, #3b82f6 0%, #2563eb 100%);
}

.warning-card::before {
  background: linear-gradient(90deg, #f59e0b 0%, #d97706 100%);
}

.error-card::before {
  background: linear-gradient(90deg, #ef4444 0%, #dc2626 100%);
}

.success-card::before {
  background: linear-gradient(90deg, #10b981 0%, #059669 100%);
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.1)"');
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.info-card .stat-icon {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
}

.warning-card .stat-icon {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
}

.error-card .stat-icon {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
}

.success-card .stat-icon {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  line-height: 1.2;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
}

/* 筛选卡片 */
.filter-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
  margin-bottom: 24px;
}

.filter-content {
  padding: 8px;
}

.filter-row {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
}

.filter-row:last-child {
  margin-bottom: 0;
}

.search-input {
  flex: 1;
}

.filter-select {
  width: 200px;
}

.filter-actions {
  display: flex;
  justify-content: flex-end;
}

/* 日志卡片 */
.logs-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
}

.logs-content {
  padding: 8px;
}

.logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

.logs-info {
  display: flex;
  gap: 16px;
  align-items: center;
}

.logs-count {
  font-size: 14px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
}

.logs-time {
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "#64748b" : "#94a3b8"');
}

/* 日志列表 */
.logs-list {
  max-height: 600px;
  overflow-y: auto;
  padding: 8px;
}

.log-item {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.02)" : "rgba(0, 0, 0, 0.02)"');
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 8px;
  transition: all 0.2s ease;
  border-left: 3px solid transparent;
}

.log-item:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.04)" : "rgba(0, 0, 0, 0.04)"');
}

.log-item.log-info {
  border-left-color: #3b82f6;
}

.log-item.log-warning {
  border-left-color: #f59e0b;
}

.log-item.log-error {
  border-left-color: #ef4444;
}

.log-item.log-success {
  border-left-color: #10b981;
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.log-time {
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.log-content {
  font-size: 13px;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  line-height: 1.5;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  word-break: break-all;
}

.log-text {
  white-space: pre-wrap;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  padding: 40px 16px;
  text-align: center;
}

.empty-icon {
  color: v-bind('themeStore.isDark ? "#4b5563" : "#9ca3af"');
  margin-bottom: 12px;
  opacity: 0.6;
}

.empty-title {
  font-size: 20px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0 0 12px 0;
}

.empty-desc {
  font-size: 14px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0 0 24px 0;
  line-height: 1.5;
  max-width: 400px;
}

.empty-btn {
  height: 42px;
  padding: 0 24px;
  font-weight: 600;
  border-radius: 10px;
  transition: all 0.2s ease;
}

.empty-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .logs-page {
    padding: 16px;
  }

  .header-content {
    flex-direction: column;
    gap: 20px;
    padding: 16px;
  }

  .header-left {
    width: 100%;
  }

  .header-actions {
    width: 100%;
    justify-content: center;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .stat-content {
    padding: 16px;
    gap: 12px;
  }

  .stat-icon {
    width: 40px;
    height: 40px;
  }

  .stat-value {
    font-size: 20px;
  }

  .filter-row {
    flex-direction: column;
    gap: 12px;
  }

  .filter-select {
    width: 100%;
  }

  .filter-actions {
    justify-content: center;
  }

  .logs-list {
    max-height: 400px;
  }

  .log-item {
    padding: 10px 12px;
  }

  .log-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
}

@media (max-width: 480px) {
  .logs-page {
    padding: 12px;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .header-content {
    padding: 16px;
  }

  .page-title {
    font-size: 20px;
  }

  .page-subtitle {
    font-size: 13px;
  }

  .log-item {
    padding: 8px 10px;
  }

  .log-content {
    font-size: 12px;
  }

  .empty-state {
    padding: 40px 16px;
    min-height: 300px;
  }

  .empty-title {
    font-size: 18px;
  }

  .empty-desc {
    font-size: 13px;
  }
}

/* 滚动条美化 */
.logs-list::-webkit-scrollbar {
  width: 6px;
}

.logs-list::-webkit-scrollbar-track {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.04)" : "rgba(0, 0, 0, 0.04)"');
  border-radius: 3px;
}

.logs-list::-webkit-scrollbar-thumb {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.2)" : "rgba(0, 0, 0, 0.2)"');
  border-radius: 3px;
}

.logs-list::-webkit-scrollbar-thumb:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.3)" : "rgba(0, 0, 0, 0.3)"');
}

/* 移除 Naive UI 组件内部样式覆盖，使用官方主题系统 */

/* 高亮样式 */
mark {
  background: rgba(91, 76, 253, 0.2) !important;
  color: #5b4cfd !important;
  padding: 2px 4px !important;
  border-radius: 4px !important;
  font-weight: 600 !important;
}
</style>