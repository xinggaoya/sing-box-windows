<template>
  <div class="log-view">
    <!-- 主要内容区 -->
    <div class="page-container">
      <!-- 页面标题栏 -->
      <div class="page-header">
        <div class="header-content">
          <div class="title-section">
            <h1 class="page-title">{{ t('log.title') }}</h1>
            <div class="title-divider"></div>
            <n-tag type="primary" size="medium" class="log-count-tag" round>
              {{ displayedLogs.length }}/{{ totalLogs }} {{ t('log.records') }}
            </n-tag>
          </div>
          <div class="header-actions">
            <n-switch v-model:value="autoScroll" size="large" class="auto-scroll-switch" round>
              <template #checked>{{ t('log.autoScroll') }}</template>
              <template #unchecked>{{ t('log.manualScroll') }}</template>
            </n-switch>

            <n-select
              v-model:value="filterType"
              :options="logTypeOptions"
              :placeholder="t('log.filterType')"
              size="large"
              class="log-filter-select"
            />

            <div class="action-buttons">
              <n-tooltip trigger="hover" placement="top">
                <template #trigger>
                  <n-button
                    type="error"
                    ghost
                    circle
                    size="large"
                    @click="clearLogs"
                    :disabled="!displayedLogs.length"
                    class="action-btn clear-btn"
                  >
                    <template #icon>
                      <n-icon><trash-outline /></n-icon>
                    </template>
                  </n-button>
                </template>
                {{ t('log.clear') }}
              </n-tooltip>

              <n-tooltip trigger="hover" placement="top">
                <template #trigger>
                  <n-button
                    type="info"
                    ghost
                    circle
                    size="large"
                    @click="copyLogs"
                    :disabled="!displayedLogs.length"
                    class="action-btn copy-btn"
                  >
                    <template #icon>
                      <n-icon><copy-outline /></n-icon>
                    </template>
                  </n-button>
                </template>
                {{ t('log.copy') }}
              </n-tooltip>

              <n-tooltip trigger="hover" placement="top">
                <template #trigger>
                  <n-button
                    type="success"
                    ghost
                    circle
                    size="large"
                    @click="exportLogs"
                    :disabled="!displayedLogs.length"
                    class="action-btn export-btn"
                  >
                    <template #icon>
                      <n-icon><download-outline /></n-icon>
                    </template>
                  </n-button>
                </template>
                {{ t('log.export') }}
              </n-tooltip>
            </div>
          </div>
        </div>
      </div>

      <!-- 搜索统计区 -->
      <div class="search-stats-section">
        <div class="search-card">
          <n-input
            v-model:value="searchQuery"
            :placeholder="t('log.searchLogs')"
            clearable
            class="search-input"
            size="large"
          >
            <template #prefix>
              <n-icon><search-outline /></n-icon>
            </template>
          </n-input>
        </div>

        <div class="stats-grid">
          <div
            v-for="(count, type) in logTypeCounts"
            :key="type"
            class="stat-card"
            :class="`stat-${type}`"
          >
            <div class="stat-icon">
              <n-icon size="24">
                <information-circle-outline v-if="type === 'info'" />
                <warning-outline v-else-if="type === 'warning'" />
                <alert-circle-outline v-else-if="type === 'error'" />
                <checkmark-circle-outline v-else-if="type === 'success'" />
              </n-icon>
            </div>
            <div class="stat-content">
              <div class="stat-label">{{ t(`log.types.${type}`) }}</div>
              <div class="stat-value">{{ count }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 日志内容区 -->
      <div class="log-content-section">
        <div class="log-content-wrapper">
          <div v-if="displayedLogs.length" class="log-list-container">
            <n-virtual-list
              ref="virtualListRef"
              class="log-virtual-list"
              :items="formattedLogs"
              :item-size="80"
              :show-scrollbar="true"
              container-style="max-height: calc(100vh - 400px); overflow: auto;"
              @scroll="handleVirtualScroll"
            >
              <template #default="{ item }">
                <div class="log-item" :key="item.key" :class="`log-item-${item.type}`">
                  <div class="log-item-indicator"></div>
                  <div class="log-item-content">
                    <div class="log-item-header">
                      <n-tag :type="getLogTagType(item.type)" size="small" round class="log-tag">
                        {{ t(`log.types.${item.type}`) }}
                      </n-tag>
                      <span class="log-time">{{ formatTime(item.timestamp, true) }}</span>
                    </div>
                    <div class="log-message-container">
                      <div class="log-message" :class="getLogClass(item.type)">
                        <n-ellipsis
                          v-if="item.payload.length > 150"
                          :line-clamp="2"
                          expand-trigger="click"
                        >
                          <template
                            v-if="
                              searchQuery &&
                              item.payload.toLowerCase().includes(searchQuery.toLowerCase())
                            "
                          >
                            <highlight-text :text="item.payload" :keyword="searchQuery" />
                          </template>
                          <template v-else>
                            {{ item.payload }}
                          </template>
                        </n-ellipsis>
                        <template v-else>
                          <template
                            v-if="
                              searchQuery &&
                              item.payload.toLowerCase().includes(searchQuery.toLowerCase())
                            "
                          >
                            <highlight-text :text="item.payload" :keyword="searchQuery" />
                          </template>
                          <template v-else>
                            {{ item.payload }}
                          </template>
                        </template>
                      </div>
                    </div>
                  </div>
                </div>
              </template>
            </n-virtual-list>
          </div>

          <div v-else class="empty-state">
            <div class="empty-content">
              <n-icon size="48" class="empty-icon">
                <document-text-outline />
              </n-icon>
              <h3 class="empty-title">
                {{ searchActive && !displayedLogs.length ? '无搜索结果' : '暂无日志' }}
              </h3>
              <p class="empty-description">
                {{
                  searchActive && !displayedLogs.length
                    ? '尝试调整搜索条件或筛选器'
                    : '当前没有日志记录'
                }}
              </p>
            </div>
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
.log-view {
  min-height: 100vh;
  background: var(--n-color-embedded);
  padding: 0;
}

.page-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: calc(100vh - 32px);
}

/* 页面标题栏 */
.page-header {
  background: var(--n-card-color);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  padding: 16px 24px;
  box-shadow: var(--n-box-shadow-2);
  border: 1px solid var(--n-border-color);
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.title-section {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.page-title {
  font-size: 1.5rem;
  font-weight: 700;
  margin: 0;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 50%, #009a1a 100%);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.title-divider {
  width: 3px;
  height: 24px;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 50%, #009a1a 100%);
  border-radius: 2px;
}

.log-count-tag {
  font-weight: 600;
  padding: 6px 12px;
  font-size: 0.8rem;
  box-shadow: 0 3px 12px rgba(64, 128, 255, 0.2);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.auto-scroll-switch {
  min-width: 140px;
  font-weight: 500;
}

.log-filter-select {
  min-width: 160px;
}

.log-filter-select :deep(.n-base-selection) {
  border-radius: 12px;
  border: 2px solid var(--n-border-color);
  transition: all 0.3s ease;
}

.log-filter-select :deep(.n-base-selection:hover) {
  border-color: rgba(64, 128, 255, 0.3);
  transform: translateY(-1px);
}

.log-filter-select :deep(.n-base-selection.n-base-selection--focus) {
  border-color: #4080ff;
  box-shadow: 0 0 0 3px rgba(64, 128, 255, 0.1);
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.action-btn {
  width: 40px;
  height: 40px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border-width: 2px;
}

.action-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}

.clear-btn:hover:not(:disabled) {
  box-shadow: 0 8px 24px rgba(245, 63, 63, 0.3);
}

.copy-btn:hover:not(:disabled) {
  box-shadow: 0 8px 24px rgba(64, 128, 255, 0.3);
}

.export-btn:hover:not(:disabled) {
  box-shadow: 0 8px 24px rgba(0, 180, 42, 0.3);
}

/* 搜索统计区 */
.search-stats-section {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 12px;
  align-items: start;
}

.search-card {
  background: var(--n-card-color);
  backdrop-filter: blur(20px);
  border-radius: 14px;
  padding: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
}

.search-input {
  width: 100%;
}

.search-input :deep(.n-input) {
  border-radius: 12px;
  border: 2px solid var(--n-border-color);
  transition: all 0.3s ease;
}

.search-input :deep(.n-input:hover) {
  border-color: rgba(64, 128, 255, 0.3);
}

.search-input :deep(.n-input.n-input--focus) {
  border-color: #4080ff;
  box-shadow: 0 0 0 3px rgba(64, 128, 255, 0.1);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
  min-width: 400px;
}

.stat-card {
  background: var(--n-card-color);
  backdrop-filter: blur(20px);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.stat-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  border-radius: 12px 12px 0 0;
}

.stat-card.stat-info::before {
  background: linear-gradient(90deg, #909399 0%, #7b7e83 100%);
}

.stat-card.stat-warning::before {
  background: linear-gradient(90deg, #ff7d00 0%, #d66600 100%);
}

.stat-card.stat-error::before {
  background: linear-gradient(90deg, #f53f3f 0%, #cb2a2a 100%);
}

.stat-card.stat-success::before {
  background: linear-gradient(90deg, #00b42a 0%, #009a1a 100%);
}

.stat-card:hover {
  transform: translateY(-3px);
  box-shadow:
    0 12px 40px rgba(0, 0, 0, 0.12),
    0 4px 8px rgba(0, 0, 0, 0.08);
}

.stat-icon {
  width: 32px;
  height: 32px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
}

.stat-info .stat-icon {
  background: linear-gradient(135deg, #909399 0%, #7b7e83 100%);
  color: white;
  box-shadow: 0 6px 20px rgba(144, 147, 153, 0.3);
}

.stat-warning .stat-icon {
  background: linear-gradient(135deg, #ff7d00 0%, #d66600 100%);
  color: white;
  box-shadow: 0 6px 20px rgba(255, 125, 0, 0.3);
}

.stat-error .stat-icon {
  background: linear-gradient(135deg, #f53f3f 0%, #cb2a2a 100%);
  color: white;
  box-shadow: 0 6px 20px rgba(245, 63, 63, 0.3);
}

.stat-success .stat-icon {
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
  color: white;
  box-shadow: 0 6px 20px rgba(0, 180, 42, 0.3);
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--n-text-color-3);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 2px;
}

.stat-value {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--n-text-color-1);
  line-height: 1.2;
}

/* 日志内容区 */
.log-content-section {
  background: var(--n-card-color);
  backdrop-filter: blur(20px);
  border-radius: 14px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  overflow: hidden;
  flex: 1;
  min-height: 350px;
}

.log-content-wrapper {
  height: 100%;
  position: relative;
  display: flex;
  flex-direction: column;
}

.log-list-container {
  flex: 1;
  padding: 12px;
}

.log-virtual-list {
  height: calc(100vh - 350px);
  min-height: 250px;
}

.log-item {
  background: var(--n-color-embedded-popover);
  border-radius: 10px;
  padding: 12px 16px;
  margin-bottom: 8px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
  display: flex;
  gap: 12px;
  border: 1px solid var(--n-border-color);
}

.log-item:hover {
  transform: translateX(4px);
  background: var(--n-color-base);
  box-shadow: var(--n-box-shadow-2);
}

.log-item-indicator {
  width: 4px;
  border-radius: 2px;
  flex-shrink: 0;
}

.log-item-info .log-item-indicator {
  background: linear-gradient(135deg, #909399 0%, #7b7e83 100%);
}

.log-item-warning .log-item-indicator {
  background: linear-gradient(135deg, #ff7d00 0%, #d66600 100%);
}

.log-item-error .log-item-indicator {
  background: linear-gradient(135deg, #f53f3f 0%, #cb2a2a 100%);
}

.log-item-success .log-item-indicator {
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
}

.log-item-content {
  flex: 1;
  min-width: 0;
}

.log-item-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.log-tag {
  flex-shrink: 0;
  font-weight: 600;
  font-size: 0.75rem;
  min-width: 60px;
  text-align: center;
}

.log-time {
  flex-shrink: 0;
  color: var(--n-text-color-3);
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 0.8rem;
  font-weight: 500;
  background: var(--n-color-embedded);
  padding: 4px 8px;
  border-radius: 6px;
}

.log-message-container {
  padding-left: 0;
}

.log-message {
  word-break: break-word;
  font-size: 0.875rem;
  line-height: 1.6;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
}

.log-info {
  color: #7b7e83;
}

.log-warning {
  color: #d66600;
}

.log-error {
  color: #cb2a2a;
}

.log-success {
  color: #009a1a;
}

/* 空状态 */
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
}

.empty-content {
  text-align: center;
  max-width: 400px;
}

.empty-icon {
  color: rgba(156, 163, 175, 0.6);
  margin-bottom: 24px;
}

.empty-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0 0 12px 0;
}

.empty-description {
  font-size: 0.875rem;
  color: var(--n-text-color-3);
  margin: 0;
  line-height: 1.5;
}

/* 深色模式样式会通过CSS变量自动应用 */

/* 文本颜色和样式会通过CSS变量自动适配暗色模式 */

/* 响应式设计 */
@media (max-width: 1024px) {
  .search-stats-section {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
    min-width: unset;
  }
}

@media (max-width: 768px) {
  .page-container {
    padding: 12px 8px;
    gap: 12px;
  }

  .page-header {
    padding: 12px 16px;
    border-radius: 12px;
  }

  .header-content {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .title-section {
    justify-content: center;
    text-align: center;
  }

  .page-title {
    font-size: 1.25rem;
  }

  .header-actions {
    flex-direction: column;
    gap: 8px;
  }

  .action-buttons {
    justify-content: center;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .stat-card {
    padding: 8px;
    border-radius: 10px;
  }

  .stat-icon {
    width: 28px;
    height: 28px;
  }

  .log-content-section {
    border-radius: 12px;
  }

  .log-virtual-list {
    height: calc(100vh - 380px);
  }

  .log-item {
    padding: 8px 12px;
    border-radius: 8px;
  }
}

@media (max-width: 480px) {
  .page-container {
    padding: 12px 8px;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .stat-card {
    padding: 16px;
  }

  .log-filter-select {
    min-width: 120px;
  }

  .auto-scroll-switch {
    min-width: 120px;
  }
}

/* 滚动条样式 */
:deep(.n-scrollbar-rail) {
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.05);
}

:deep(.n-scrollbar-rail--vertical) {
  width: 8px;
}

:deep(.n-scrollbar-rail--horizontal) {
  height: 8px;
}

:deep(.n-scrollbar-content) {
  border-radius: 8px;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 100%);
  transition: all 0.3s ease;
}

:deep(.n-scrollbar-content:hover) {
  background: linear-gradient(135deg, #6699ff 0%, #4080ff 100%);
}

/* 动画效果 */
@keyframes slide-up {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.log-view {
  animation: slide-up 0.4s ease;
}
</style>
