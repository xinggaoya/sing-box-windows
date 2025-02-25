<template>
  <div class="log-container">
    <!-- 日志头部卡片 -->
    <n-card class="log-header-card" :bordered="false">
      <n-space align="center" justify="space-between">
        <div class="title-container">
          <n-h3 class="card-title">
            <n-icon size="24" class="card-icon">
              <document-text-outline />
            </n-icon>
            日志查看
          </n-h3>
          <n-tag :bordered="false" type="info" size="medium" class="log-count-tag">
            {{ displayedLogs.length }}/{{ totalLogs }} 条记录
          </n-tag>
        </div>
        <n-space :size="12">
          <n-switch v-model:value="autoScroll" size="medium" class="auto-scroll-switch">
            <template #checked>自动滚动</template>
            <template #unchecked>手动滚动</template>
          </n-switch>

          <n-select
            v-model:value="filterType"
            :options="logTypeOptions"
            placeholder="筛选日志类型"
            size="medium"
            style="width: 120px"
            class="log-filter-select"
          />

          <n-space>
            <n-tooltip trigger="hover" placement="top">
              <template #trigger>
                <n-button
                  quaternary
                  circle
                  size="medium"
                  @click="clearLogs"
                  :disabled="!displayedLogs.length"
                  class="log-action-button"
                >
                  <template #icon>
                    <n-icon><trash-outline /></n-icon>
                  </template>
                </n-button>
              </template>
              清空日志
            </n-tooltip>

            <n-tooltip trigger="hover" placement="top">
              <template #trigger>
                <n-button
                  quaternary
                  circle
                  size="medium"
                  @click="copyLogs"
                  :disabled="!displayedLogs.length"
                  class="log-action-button"
                >
                  <template #icon>
                    <n-icon><copy-outline /></n-icon>
                  </template>
                </n-button>
              </template>
              复制日志
            </n-tooltip>

            <n-tooltip trigger="hover" placement="top">
              <template #trigger>
                <n-button
                  quaternary
                  circle
                  size="medium"
                  @click="exportLogs"
                  :disabled="!displayedLogs.length"
                  class="log-action-button"
                >
                  <template #icon>
                    <n-icon><download-outline /></n-icon>
                  </template>
                </n-button>
              </template>
              导出日志
            </n-tooltip>
          </n-space>
        </n-space>
      </n-space>
    </n-card>

    <!-- 日志内容卡片 -->
    <n-card class="log-content-card" :bordered="false">
      <n-scrollbar ref="scrollbarRef" class="log-scrollbar" trigger="none" @scroll="handleScroll">
        <div class="log-content">
          <div v-for="(log, index) in displayedLogs" :key="log.timestamp + index" class="log-item">
            <n-tag :type="getLogTagType(log.type)" size="small" round class="log-tag">
              {{ log.type }}
            </n-tag>
            <span class="log-time">{{ formatTime(log.timestamp, true) }}</span>
            <span class="log-message" :class="getLogClass(log.type)">{{ log.payload }}</span>
          </div>
          <n-empty v-if="!displayedLogs.length" description="暂无日志记录" class="log-empty" />
        </div>
      </n-scrollbar>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { useInfoStore } from '@/stores/infoStore'
import { onMounted, ref, computed, onUnmounted, watch, nextTick } from 'vue'
import { useMessage } from 'naive-ui'
import { TrashOutline, CopyOutline, DownloadOutline, DocumentTextOutline } from '@vicons/ionicons5'

interface Log {
  type: string
  payload: string
  timestamp: number
}

interface ScrollbarInstance {
  scrollTo: (options: ScrollToOptions) => void
  containerRef: { scrollHeight: number }
}

interface ScrollEvent {
  target: {
    scrollTop: number
    scrollHeight: number
    clientHeight: number
  }
}

const message = useMessage()
const infoStore = useInfoStore()
const scrollbarRef = ref(null)
const autoScroll = ref(true)
const filterType = ref<string | null>(null)
const displayedLogs = ref<Log[]>([])

// 监听日志变化
watch(
  () => infoStore.logs,
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
watch(filterType, () => {
  updateDisplayedLogs()
})

const totalLogs = computed(() => {
  return filterType.value
    ? infoStore.logs.filter((log: Log) => log.type === filterType.value).length
    : infoStore.logs.length
})

const logTypeOptions = [
  { label: '全部', value: null },
  { label: '信息', value: 'info' },
  { label: '警告', value: 'warning' },
  { label: '错误', value: 'error' },
  { label: '成功', value: 'success' },
]

// 更新显示的日志
const updateDisplayedLogs = () => {
  displayedLogs.value = filterType.value
    ? infoStore.logs.filter((log) => log.type === filterType.value)
    : infoStore.logs
}

// 滚动到底部
const scrollToBottom = () => {
  const scrollbarElement = scrollbarRef.value
  if (!scrollbarElement) return

  // 使用 nextTick 确保 DOM 已更新
  nextTick(() => {
    // 使用类型断言确保类型安全
    const scrollbar = scrollbarElement as unknown as ScrollbarInstance
    if (scrollbar?.containerRef) {
      scrollbar.scrollTo({
        top: scrollbar.containerRef.scrollHeight,
        behavior: 'smooth',
      })
    }
  })
}

// 处理滚动
const handleScroll = (e: ScrollEvent) => {
  const { scrollTop, scrollHeight, clientHeight } = e.target
  // 如果用户向上滚动，关闭自动滚动
  if (scrollHeight - scrollTop - clientHeight > 100) {
    autoScroll.value = false
  }
}

onMounted(() => {
  updateDisplayedLogs()
  if (autoScroll.value) {
    scrollToBottom()
  }
})

const clearLogs = () => {
  infoStore.logs = []
  displayedLogs.value = []
  message.success('日志已清空')
}

const copyLogs = () => {
  const logText = displayedLogs.value
    .map((log) => `[${formatTime(log.timestamp)}] [${log.type}] ${log.payload}`)
    .join('\n')
  navigator.clipboard.writeText(logText)
  message.success('日志已复制到剪贴板')
}

const exportLogs = () => {
  const logText = displayedLogs.value
    .map((log) => `[${formatTime(log.timestamp)}] [${log.type}] ${log.payload}`)
    .join('\n')
  const blob = new Blob([logText], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `sing-box-logs-${formatTime(Date.now(), true).replace(':', '-')}.txt`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  message.success('日志已导出')
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
</script>

<style scoped>
.log-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 8px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: calc(100vh - 100px);
  animation: slide-up 0.4s ease;
}

.log-header-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.log-header-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.title-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-weight: 600;
}

.card-icon {
  color: var(--primary-color);
}

.log-count-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
  background-color: rgba(144, 147, 153, 0.12);
  color: var(--n-text-color-2);
}

.log-content-card {
  flex: 1;
  border-radius: 16px;
  min-height: 300px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.log-content-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.log-scrollbar {
  height: 100%;
  padding: 0 4px;
}

.log-content {
  padding: 8px 4px;
}

.log-item {
  padding: 8px 12px;
  border-radius: 8px;
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.3s ease;
  border-left: 3px solid transparent;
  border: 1px solid var(--n-border-color);
}

.log-item:hover {
  transform: translateX(2px);
}

.log-tag {
  flex-shrink: 0;
  min-width: 50px;
  text-align: center;
  font-weight: 500;
}

.log-time {
  flex-shrink: 0;
  color: var(--n-text-color-3);
  font-family: monospace;
  font-size: 13px;
  min-width: 80px;
}

.log-message {
  word-break: break-word;
  font-size: 14px;
  line-height: 1.6;
}

.log-info {
  color: var(--n-text-color-1);
}

.log-warning {
  color: var(--warning-color);
}

.log-error {
  color: var(--error-color);
}

.log-success {
  color: var(--success-color);
}

.log-action-button {
  transition: all 0.3s ease;
}

.log-action-button:hover:not(:disabled) {
  transform: translateY(-2px);
  color: var(--primary-color);
  background-color: rgba(64, 128, 255, 0.1);
}

.auto-scroll-switch {
  min-width: 100px;
}

.log-filter-select {
  transition: all 0.3s ease;
}

.log-filter-select:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-focus);
}

.log-empty {
  margin: 60px 0;
  opacity: 0.8;
}

.log-item:has(.log-info) {
  border-left-color: var(--info-color);
}

.log-item:has(.log-warning) {
  border-left-color: var(--warning-color);
}

.log-item:has(.log-error) {
  border-left-color: var(--error-color);
}

.log-item:has(.log-success) {
  border-left-color: var(--success-color);
}
</style>
