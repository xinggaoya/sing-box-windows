<template>
  <div class="log-view">
    <n-card size="small" style="margin-bottom: 8px">
      <n-space align="center" justify="space-between">
        <n-space align="center">
          <n-h3 style="margin: 0">运行日志</n-h3>
          <n-text depth="3">{{ displayedLogs.length }}/{{ totalLogs }} 条记录</n-text>
        </n-space>
        <n-space>
          <n-switch v-model:value="autoScroll">
            <template #checked>自动滚动</template>
            <template #unchecked>手动滚动</template>
          </n-switch>
          <n-select
            v-model:value="filterType"
            :options="logTypeOptions"
            placeholder="筛选日志类型"
            style="width: 140px"
          />
          <n-button
            quaternary
            circle
            type="primary"
            @click="clearLogs"
            :disabled="!displayedLogs.length"
          >
            <template #icon>
              <n-icon><TrashOutline /></n-icon>
            </template>
          </n-button>
          <n-button
            quaternary
            circle
            type="primary"
            @click="copyLogs"
            :disabled="!displayedLogs.length"
          >
            <template #icon>
              <n-icon><CopyOutline /></n-icon>
            </template>
          </n-button>
          <n-button
            quaternary
            circle
            type="primary"
            @click="exportLogs"
            :disabled="!displayedLogs.length"
          >
            <template #icon>
              <n-icon><DownloadOutline /></n-icon>
            </template>
          </n-button>
        </n-space>
      </n-space>
    </n-card>

    <n-card size="small" class="log-card">
      <n-scrollbar ref="scrollbarRef" class="log-scrollbar" trigger="none" @scroll="handleScroll">
        <div class="log-content">
          <div v-for="(log, index) in displayedLogs" :key="log.timestamp + index" class="log-item">
            <n-tag :type="getLogTagType(log.type)" size="small" round>{{ log.type }}</n-tag>
            <span class="log-time">{{ formatTime(log.timestamp, true) }}</span>
            <span class="log-message" :class="getLogClass(log.type)">{{ log.payload }}</span>
          </div>
          <n-empty v-if="!displayedLogs.length" description="暂无日志记录" />
        </div>
      </n-scrollbar>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { useInfoStore } from '@/stores/infoStore'
import { onMounted, ref, computed, onUnmounted, watch, nextTick } from 'vue'
import { useMessage } from 'naive-ui'
import {
  TrashOutline,
  CopyOutline,
  DownloadOutline,
  InformationCircleOutline,
  WarningOutline,
  CloseCircleOutline,
  CheckmarkCircleOutline,
} from '@vicons/ionicons5'

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
.log-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.log-card {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.log-scrollbar {
  flex: 1;
}

.log-content {
  padding: 8px;
}

.log-item {
  font-family: 'Consolas', monospace;
  padding: 4px 8px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 1px solid var(--divider-color);
}

.log-time {
  color: var(--text-color-3);
  font-size: 0.9em;
  min-width: 80px;
}

.log-message {
  flex: 1;
  white-space: pre-wrap;
  word-break: break-all;
}

.log-info {
  color: var(--info-color);
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
</style>
