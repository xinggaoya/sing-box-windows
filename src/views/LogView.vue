<template>
  <div class="ultra-logs">
    <!-- 紧凑工具栏 -->
    <div class="logs-toolbar">
      <div class="toolbar-left">
        <div class="toolbar-icon">
          <n-icon size="16">
            <DocumentTextOutline />
          </n-icon>
        </div>
        <div class="toolbar-info">
          <span class="toolbar-title">{{ t('log.title') }}</span>
          <span class="toolbar-stats">{{ displayedLogs.length }}/{{ totalLogs }} {{ t('log.records') }}</span>
        </div>
      </div>

      <div class="toolbar-right">
        <div class="toolbar-controls">
          <n-switch
            v-model:value="autoScroll"
            size="small"
            class="auto-scroll-switch"
          >
            <template #checked>{{ t('log.autoScroll') }}</template>
            <template #unchecked>{{ t('log.manualScroll') }}</template>
          </n-switch>

          <n-select
            v-model:value="filterType"
            :options="logTypeOptions"
            :placeholder="t('log.level')"
            clearable
            size="small"
            class="log-filter-select"
          />

          <n-button-group size="small">
            <n-tooltip trigger="hover" placement="top">
              <template #trigger>
                <n-button
                  type="error"
                  @click="clearLogs"
                  :disabled="!displayedLogs.length"
                  class="action-btn clear-btn"
                >
                  <template #icon>
                    <n-icon size="14"><TrashOutline /></n-icon>
                  </template>
                </n-button>
              </template>
              {{ t('log.clear') }}
            </n-tooltip>

            <n-tooltip trigger="hover" placement="top">
              <template #trigger>
                <n-button
                  type="info"
                  @click="copyLogs"
                  :disabled="!displayedLogs.length"
                  class="action-btn copy-btn"
                >
                  <template #icon>
                    <n-icon size="14"><CopyOutline /></n-icon>
                  </template>
                </n-button>
              </template>
              {{ t('log.copy') }}
            </n-tooltip>

            <n-tooltip trigger="hover" placement="top">
              <template #trigger>
                <n-button
                  type="success"
                  @click="exportLogs"
                  :disabled="!displayedLogs.length"
                  class="action-btn export-btn"
                >
                  <template #icon>
                    <n-icon size="14"><DownloadOutline /></n-icon>
                  </template>
                </n-button>
              </template>
              {{ t('log.export') }}
            </n-tooltip>
          </n-button-group>
        </div>
      </div>
    </div>

    <!-- 统计面板 -->
    <div class="stats-panel">
      <div
        v-for="(count, type) in logTypeCounts"
        :key="type"
        class="stat-orb"
        :class="`orb-${type}`"
      >
        <div class="orb-icon">
          <n-icon size="14">
            <InformationCircleOutline v-if="type === 'info'" />
            <WarningOutline v-else-if="type === 'warning'" />
            <AlertCircleOutline v-else-if="type === 'error'" />
            <CheckmarkCircleOutline v-else-if="type === 'success'" />
          </n-icon>
        </div>
        <div class="orb-content">
          <div class="orb-value">{{ count }}</div>
          <div class="orb-label">{{ t(`log.types.${type}`) }}</div>
        </div>
      </div>
    </div>

    <!-- 日志内容区域 -->
    <div class="logs-content">
      <!-- 搜索区域 -->
      <div class="search-section">
        <div class="search-input-group">
          <n-input
            v-model:value="searchQuery"
            :placeholder="t('log.searchLogs')"
            clearable
            size="small"
            class="search-input"
          >
            <template #prefix>
              <n-icon size="14">
                <SearchOutline />
              </n-icon>
            </template>
          </n-input>

          <div class="search-stats">
            <n-tag v-if="totalLogs > 0" type="info" size="tiny" round>
              {{ t('log.totalCount', { count: totalLogs }) }}
            </n-tag>
            <n-tag v-if="(searchQuery || filterType) && displayedLogs.length > 0" type="success" size="tiny" round>
              {{ t('log.matchCount', { count: displayedLogs.length }) }}
            </n-tag>
          </div>
        </div>
      </div>

      <!-- 日志列表 -->
      <div class="logs-list">
        <div v-if="displayedLogs.length" class="logs-container">
          <n-virtual-list
            ref="virtualListRef"
            class="log-virtual-list"
            :items="formattedLogs"
            :item-size="60"
            :show-scrollbar="true"
            @scroll="handleVirtualScroll"
          >
            <template #default="{ item }">
              <div class="log-item" :key="item.key" :class="`log-item-${item.type}`">
                <div class="log-indicator"></div>
                <div class="log-content">
                  <div class="log-header">
                    <div class="log-type-badge" :class="`type-${item.type}`">
                      {{ t(`log.types.${item.type}`) }}
                    </div>
                    <div class="log-time">{{ formatTime(item.timestamp, true) }}</div>
                  </div>
                  <div class="log-message">
                    <template
                      v-if="searchQuery && item.payload.toLowerCase().includes(searchQuery.toLowerCase())"
                    >
                      <highlight-text :text="item.payload" :keyword="searchQuery" />
                    </template>
                    <template v-else>
                      {{ item.payload }}
                    </template>
                  </div>
                </div>
              </div>
            </template>
          </n-virtual-list>
        </div>

        <!-- 空状态 -->
        <div v-else class="empty-state">
          <div class="empty-icon">
            <n-icon size="32">
              <DocumentTextOutline />
            </n-icon>
          </div>
          <div class="empty-title">
            {{
              searchActive && !displayedLogs.length
                ? t('log.noSearchResults')
                : t('log.noLogs')
            }}
          </div>
          <div class="empty-desc">
            {{
              searchActive && !displayedLogs.length
                ? t('log.adjustSearchFilters')
                : t('log.noLogRecords')
            }}
          </div>
        </div>
      </div>
    </div>
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
import type { VirtualListInst } from 'naive-ui'
import { useI18n } from 'vue-i18n'

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
      if (!props.keyword) return h('span', {}, props.text)

      const keyword = props.keyword.toLowerCase()
      const text = props.text
      const parts = []
      let lastIndex = 0

      let index = text.toLowerCase().indexOf(keyword)
      while (index !== -1) {
        // 添加前面的文本
        if (index > lastIndex) {
          parts.push(h('span', {}, text.substring(lastIndex, index)))
        }

        // 添加高亮部分
        parts.push(
          h(
            'span',
            {
              style: {
                backgroundColor: 'rgba(var(--primary-color), 0.1)',
                fontWeight: 'bold',
                padding: '0 2px',
                borderRadius: '2px',
              },
            },
            text.substring(index, index + keyword.length),
          ),
        )

        lastIndex = index + keyword.length
        index = text.toLowerCase().indexOf(keyword, lastIndex)
      }

      // 添加剩余部分
      if (lastIndex < text.length) {
        parts.push(h('span', {}, text.substring(lastIndex)))
      }

      return h('span', {}, parts)
    }
  },
})

const message = useMessage()
const logStore = useLogStore()
const virtualListRef = ref<VirtualListInst | null>(null)
const autoScroll = ref(true)
const filterType = ref<string | null>(null)
const searchQuery = ref('')
const displayedLogs = ref<Log[]>([])
const { t } = useI18n()

// 判断是否处于搜索状态
const searchActive = computed(() => {
  return searchQuery.value.length > 0
})

// 格式化日志数据，添加key属性以适配虚拟列表
const formattedLogs = computed<FormattedLog[]>(() => {
  return displayedLogs.value.map((log, index) => ({
    ...log,
    key: `${log.timestamp}-${index}`,
  }))
})

// 统计各日志类型的数量
const logTypeCounts = computed(() => {
  const counts: Record<string, number> = {
    info: 0,
    warning: 0,
    error: 0,
    success: 0,
  }

  logStore.logs.forEach((log) => {
    if (counts[log.type] !== undefined) {
      counts[log.type]++
    }
  })

  return counts
})

// 处理虚拟列表的滚动事件
const handleVirtualScroll = (e: Event) => {
  const target = e.target as HTMLElement
  if (!target) return

  const { scrollTop, scrollHeight, clientHeight } = target

  // 如果用户向上滚动超过一定距离，关闭自动滚动
  if (scrollHeight - scrollTop - clientHeight > 100) {
    autoScroll.value = false
  }
}

// 更新显示的日志 - 确保此函数在watch之前定义
const updateDisplayedLogs = () => {
  let filtered = [...logStore.logs]

  // 按类型筛选
  if (filterType.value) {
    filtered = filtered.filter((log) => log.type === filterType.value)
  }

  // 按搜索关键词筛选
  if (searchQuery.value) {
    filtered = filtered.filter((log) =>
      log.payload.toLowerCase().includes(searchQuery.value.toLowerCase()),
    )
  }

  displayedLogs.value = filtered
}

// 滚动到底部
const scrollToBottom = () => {
  nextTick(() => {
    if (virtualListRef.value) {
      // 使用虚拟列表组件提供的scrollTo方法滚动到最底部
      virtualListRef.value.scrollTo({ index: displayedLogs.value.length - 1 })
    }
  })
}

// 监听日志变化 - 现在updateDisplayedLogs已在上方定义
watch(
  () => logStore.logs,
  async (newLogs) => {
    updateDisplayedLogs()
    if (autoScroll.value) {
      await nextTick()
      scrollToBottom()
    }
  },
  { deep: true },
)

// 监听筛选条件变化
watch(
  [filterType, searchQuery],
  () => {
    updateDisplayedLogs()
    nextTick(() => {
      if (autoScroll.value) {
        scrollToBottom()
      }
    })
  },
  { immediate: true },
)

// 监听自动滚动状态变化
watch(autoScroll, (newValue) => {
  if (newValue) {
    scrollToBottom()
  }
})

// 计算总日志数
const totalLogs = computed(() => {
  return logStore.logs.length
})

const logTypeOptions = [
  { label: t('log.types.all'), value: null },
  { label: t('log.types.info'), value: 'info' },
  { label: t('log.types.warning'), value: 'warning' },
  { label: t('log.types.error'), value: 'error' },
  { label: t('log.types.success'), value: 'success' },
]

const clearLogs = () => {
  // 使用store提供的方法清空日志
  logStore.clearLogs()
  displayedLogs.value = []
  message.success(t('log.clearedSuccess'))
}

const copyLogs = () => {
  const logText = displayedLogs.value
    .map((log) => `[${formatTime(log.timestamp)}] [${t(`log.types.${log.type}`)}] ${log.payload}`)
    .join('\n')
  navigator.clipboard.writeText(logText)
  message.success(t('log.copiedSuccess'))
}

const exportLogs = () => {
  const logText = displayedLogs.value
    .map((log) => `[${formatTime(log.timestamp)}] [${t(`log.types.${log.type}`)}] ${log.payload}`)
    .join('\n')
  const blob = new Blob([logText], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `sing-box-logs-${formatTime(Date.now(), true).replace(/:/g, '-')}.txt`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  message.success(t('log.exportedSuccess'))
}

const formatTime = (timestamp: number, showSeconds = false) => {
  const date = new Date(timestamp)
  const hours = date.getHours().toString().padStart(2, '0')
  const minutes = date.getMinutes().toString().padStart(2, '0')
  const seconds = date.getSeconds().toString().padStart(2, '0')
  return showSeconds ? `${hours}:${minutes}:${seconds}` : `${hours}:${minutes}`
}

const getLogTagType = (type: string): 'success' | 'warning' | 'error' | 'info' => {
  const typeMap: Record<string, 'success' | 'warning' | 'error' | 'info'> = {
    info: 'info',
    warning: 'warning',
    error: 'error',
    success: 'success',
  }
  return typeMap[type] || 'info'
}

const getLogClass = (type: string): string => {
  return `log-${type}`
}

const getLogColor = (type: string): string => {
  const colorMap: Record<string, string> = {
    info: '#2080f0',
    warning: '#f0a020',
    error: '#d03050',
    success: '#18a058',
  }
  return colorMap[type] || '#909399'
}

onMounted(async () => {
  // 设置日志监听器
  await logStore.setupLogListener()

  updateDisplayedLogs()
  nextTick(() => {
    if (autoScroll.value) {
      scrollToBottom()
    }
  })
})

// 组件卸载时清理日志监听器
onUnmounted(() => {
  logStore.cleanupListeners()
})
</script>

<style scoped>
.ultra-logs {
  padding: 16px;
  background: var(--n-color-embedded);
  min-height: calc(100vh - 36px);
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: slideFadeIn 0.4s ease-out;
}

/* 紧凑工具栏 */
.logs-toolbar {
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toolbar-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: linear-gradient(135deg, #ff7d00 0%, #d66600 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 12px rgba(255, 125, 0, 0.3);
}

.toolbar-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toolbar-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0;
}

.toolbar-stats {
  font-size: 0.75rem;
  color: var(--n-text-color-3);
  margin: 0;
}

.toolbar-right {
  display: flex;
  align-items: center;
}

.toolbar-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.auto-scroll-switch {
  min-width: 60px;
}

.log-filter-select {
  min-width: 100px;
}

.action-btn {
  height: 28px;
  padding: 0 8px;
  font-size: 0.75rem;
  transition: all 0.2s ease;
}

.action-btn:hover:not(:disabled) {
  transform: translateY(-1px);
}

.clear-btn:hover:not(:disabled) {
  box-shadow: 0 4px 12px rgba(245, 63, 63, 0.3);
}

.copy-btn:hover:not(:disabled) {
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.3);
}

.export-btn:hover:not(:disabled) {
  box-shadow: 0 4px 12px rgba(0, 180, 42, 0.3);
}

/* 统计面板 */
.stats-panel {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
}

.stat-orb {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  border-radius: 8px;
  transition: all 0.2s ease;
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.stat-orb:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.stat-orb::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  border-radius: 8px 8px 0 0;
}

.orb-info::before {
  background: linear-gradient(90deg, #909399 0%, #7b7e83 100%);
}

.orb-warning::before {
  background: linear-gradient(90deg, #ff7d00 0%, #d66600 100%);
}

.orb-error::before {
  background: linear-gradient(90deg, #f53f3f 0%, #cb2a2a 100%);
}

.orb-success::before {
  background: linear-gradient(90deg, #00b42a 0%, #009a1a 100%);
}

.orb-icon {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 500;
}

.orb-info .orb-icon {
  background: linear-gradient(135deg, #909399 0%, #7b7e83 100%);
}

.orb-warning .orb-icon {
  background: linear-gradient(135deg, #ff7d00 0%, #d66600 100%);
}

.orb-error .orb-icon {
  background: linear-gradient(135deg, #f53f3f 0%, #cb2a2a 100%);
}

.orb-success .orb-icon {
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
}

.orb-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.orb-value {
  font-size: 1rem;
  font-weight: 700;
  color: var(--n-text-color-1);
  line-height: 1.2;
}

.orb-label {
  font-size: 0.7rem;
  color: var(--n-text-color-3);
  font-weight: 500;
}

/* 日志内容区域 */
.logs-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 0;
}

/* 搜索区域 */
.search-section {
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
}

.search-input-group {
  display: flex;
  gap: 12px;
  align-items: center;
}

.search-input {
  flex: 1;
  min-width: 280px;
}

.search-input :deep(.n-input) {
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
}

.search-input :deep(.n-input:hover) {
  border-color: #ff7d00;
}

.search-input :deep(.n-input.n-input--focus) {
  border-color: #ff7d00;
  box-shadow: 0 0 0 2px rgba(255, 125, 0, 0.1);
}

.search-stats {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

/* 日志列表 */
.logs-list {
  flex: 1;
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  min-height: 0;
}

.logs-container {
  height: 100%;
  min-height: 300px;
}

.log-virtual-list {
  height: 100%;
}

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  background: var(--n-color-embedded);
  border-radius: 8px;
  margin-bottom: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
  position: relative;
  overflow: hidden;
}

.log-item:hover {
  background: var(--n-color-embedded-modal);
  border-color: #ff7d00;
  transform: translateX(2px);
  box-shadow: 0 2px 8px rgba(255, 125, 0, 0.1);
}

.log-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: transparent;
  transition: background 0.2s ease;
}

.log-item:hover::before {
  background: #ff7d00;
}

.log-indicator {
  width: 4px;
  border-radius: 2px;
  flex-shrink: 0;
  margin-top: 2px;
}

.log-item-info .log-indicator {
  background: linear-gradient(135deg, #909399 0%, #7b7e83 100%);
}

.log-item-warning .log-indicator {
  background: linear-gradient(135deg, #ff7d00 0%, #d66600 100%);
}

.log-item-error .log-indicator {
  background: linear-gradient(135deg, #f53f3f 0%, #cb2a2a 100%);
}

.log-item-success .log-indicator {
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
}

.log-content {
  flex: 1;
  min-width: 0;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.log-type-badge {
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 500;
  text-align: center;
  white-space: nowrap;
}

.type-info {
  background: rgba(144, 147, 153, 0.1);
  color: #7b7e83;
  border: 1px solid rgba(144, 147, 153, 0.2);
}

.type-warning {
  background: rgba(255, 125, 0, 0.1);
  color: #d66600;
  border: 1px solid rgba(255, 125, 0, 0.2);
}

.type-error {
  background: rgba(245, 63, 63, 0.1);
  color: #cb2a2a;
  border: 1px solid rgba(245, 63, 63, 0.2);
}

.type-success {
  background: rgba(0, 180, 42, 0.1);
  color: #009a1a;
  border: 1px solid rgba(0, 180, 42, 0.2);
}

.log-time {
  font-size: 0.7rem;
  color: var(--n-text-color-3);
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-weight: 500;
  background: var(--n-color-embedded-modal);
  padding: 2px 6px;
  border-radius: 4px;
}

.log-message {
  font-size: 0.8rem;
  line-height: 1.5;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  color: var(--n-text-color-1);
  word-break: break-word;
}

/* 高亮标记 */
.log-message :deep(span[style*="background"]) {
  background: rgba(255, 125, 0, 0.2) !important;
  color: var(--n-text-color-1) !important;
  padding: 1px 2px;
  border-radius: 2px;
  font-weight: 600;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  padding: 40px 20px;
  text-align: center;
}

.empty-icon {
  color: var(--n-text-color-disabled);
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0 0 8px 0;
}

.empty-desc {
  font-size: 0.875rem;
  color: var(--n-text-color-3);
  margin: 0 0 20px 0;
  line-height: 1.5;
  max-width: 300px;
}

/* 动画效果 */
@keyframes slideFadeIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 响应式设计 */
@media (max-width: 1024px) {
  .stats-panel {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .ultra-logs {
    padding: 12px;
    gap: 12px;
  }

  .logs-toolbar {
    padding: 10px 12px;
  }

  .toolbar-icon {
    width: 28px;
    height: 28px;
  }

  .toolbar-title {
    font-size: 0.875rem;
  }

  .toolbar-stats {
    font-size: 0.7rem;
  }

  .toolbar-controls {
    gap: 6px;
  }

  .auto-scroll-switch {
    min-width: 50px;
  }

  .log-filter-select {
    min-width: 80px;
  }

  .action-btn {
    padding: 0 6px;
    font-size: 0.7rem;
  }

  .stats-panel {
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    padding: 12px;
  }

  .stat-orb {
    padding: 8px;
    flex-direction: column;
    text-align: center;
    gap: 4px;
  }

  .orb-icon {
    width: 24px;
    height: 24px;
  }

  .orb-value {
    font-size: 0.875rem;
  }

  .orb-label {
    font-size: 0.65rem;
  }

  .search-input-group {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .search-input {
    min-width: unset;
  }

  .log-item {
    padding: 10px;
    gap: 8px;
  }

  .log-type-badge {
    font-size: 0.65rem;
    padding: 1px 4px;
  }

  .log-time {
    font-size: 0.65rem;
    padding: 1px 4px;
  }

  .log-message {
    font-size: 0.75rem;
  }
}

@media (max-width: 480px) {
  .ultra-logs {
    padding: 8px;
    gap: 8px;
  }

  .logs-toolbar {
    padding: 8px 10px;
  }

  .toolbar-left {
    gap: 8px;
  }

  .toolbar-icon {
    width: 24px;
    height: 24px;
  }

  .toolbar-title {
    font-size: 0.8rem;
  }

  .toolbar-controls {
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .search-section {
    padding: 12px;
  }

  .logs-list {
    padding: 12px;
  }

  .stats-panel {
    grid-template-columns: repeat(2, 1fr);
  }

  .log-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .log-type-badge {
    font-size: 0.6rem;
  }

  .log-message {
    font-size: 0.7rem;
  }

  .empty-state {
    padding: 32px 16px;
    min-height: 250px;
  }

  .empty-title {
    font-size: 1rem;
  }

  .empty-desc {
    font-size: 0.8rem;
  }
}

/* Naive UI 组件优化 */
:deep(.n-spin-container) {
  min-height: 200px;
}

:deep(.n-input__input-el) {
  font-size: 0.875rem !important;
}

:deep(.n-base-selection-label) {
  font-size: 0.875rem !important;
}

:deep(.n-button__content) {
  font-size: 0.75rem !important;
}

:deep(.n-scrollbar-rail) {
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.05);
}

:deep(.n-scrollbar-rail--vertical) {
  width: 6px;
}

:deep(.n-scrollbar-content) {
  border-radius: 6px;
  background: rgba(255, 125, 0, 0.3);
  transition: all 0.2s ease;
}

:deep(.n-scrollbar-content:hover) {
  background: rgba(255, 125, 0, 0.5);
}
</style>
