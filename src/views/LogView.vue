<template>
  <n-space vertical>
    <n-card>
      <n-space align="center" justify="space-between">
        <n-space align="center">
          <n-h3 style="margin: 0">运行日志</n-h3>
          <n-text depth="3">{{ displayedLogs.length }}/{{ totalLogs }} 条记录</n-text>
        </n-space>
        <n-space>
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
        </n-space>
      </n-space>
    </n-card>

    <n-card>
      <n-scrollbar 
        ref="scrollbarRef"
        style="max-height: calc(100vh - 180px)" 
        trigger="none"
        @scroll="handleScroll"
      >
        <div class="log-container">
          <n-timeline>
            <n-timeline-item
              v-for="(log, index) in displayedLogs"
              :key="log.timestamp + index"
              :type="getLogType(log.type)"
              :title="formatTime(log.timestamp)"
              :content="log.payload"
              :time="formatTime(log.timestamp, true)"
            >
              <template #icon>
                <n-icon>
                  <component :is="getLogIcon(log.type)" />
                </n-icon>
              </template>
            </n-timeline-item>
          </n-timeline>
          <n-empty
            v-if="!displayedLogs.length"
            description="暂无日志记录"
          />
        </div>
      </n-scrollbar>
    </n-card>
  </n-space>
</template>

<script setup lang="ts">
import { useInfoStore } from '@/stores/infoStore'
import { onMounted, ref, computed, onUnmounted, watch } from 'vue'
import { useMessage } from 'naive-ui'
import {
  TrashOutline,
  CopyOutline,
  InformationCircleOutline,
  WarningOutline,
  CloseCircleOutline,
  CheckmarkCircleOutline
} from '@vicons/ionicons5'

interface Log {
  type: string
  payload: string
  timestamp: number
}

const PAGE_SIZE = 50; // 每页显示的日志数量
const message = useMessage()
const infoStore = useInfoStore()
const scrollbarRef = ref(null)
const currentPage = ref(1)
const filterType = ref<string | null>(null)
const displayedLogs = ref<Log[]>([])

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
  { label: '成功', value: 'success' }
]

// 监听筛选条件变化
watch(filterType, () => {
  currentPage.value = 1;
  loadLogs();
})

// 加载日志
const loadLogs = () => {
  const start = (currentPage.value - 1) * PAGE_SIZE;
  const filteredLogs = filterType.value
    ? infoStore.logs.filter((log: Log) => log.type === filterType.value)
    : infoStore.logs;
  
  displayedLogs.value = filteredLogs
    .slice(start, start + PAGE_SIZE)
    .map((log: Log) => ({
      ...log,
      timestamp: log.timestamp || Date.now()
    }));
}

// 处理滚动
const handleScroll = (e: any) => {
  const { scrollTop, scrollHeight, clientHeight } = e.target;
  if (scrollHeight - scrollTop - clientHeight < 50 && 
      currentPage.value * PAGE_SIZE < totalLogs.value) {
    currentPage.value++;
    loadLogs();
  }
}

onMounted(() => {
  loadLogs();
})

// 清理函数
onUnmounted(() => {
  displayedLogs.value = [];
})

const clearLogs = () => {
  infoStore.logs = [];
  displayedLogs.value = [];
  currentPage.value = 1;
  message.success('日志已清空');
}

const copyLogs = () => {
  const logText = displayedLogs.value
    .map(log => `[${formatTime(log.timestamp)}] [${log.type}] ${log.payload}`)
    .join('\n');
  navigator.clipboard.writeText(logText);
  message.success('日志已复制到剪贴板');
}

const formatTime = (timestamp: number, showSeconds = false) => {
  const date = new Date(timestamp)
  const hours = date.getHours().toString().padStart(2, '0')
  const minutes = date.getMinutes().toString().padStart(2, '0')
  const seconds = date.getSeconds().toString().padStart(2, '0')
  return showSeconds
    ? `${hours}:${minutes}:${seconds}`
    : `${hours}:${minutes}`
}

const getLogType = (type: string): 'info' | 'warning' | 'error' | 'success' => {
  const typeMap: Record<string, 'info' | 'warning' | 'error' | 'success'> = {
    info: 'info',
    warning: 'warning',
    error: 'error',
    success: 'success'
  }
  return typeMap[type] || 'info'
}

const getLogIcon = (type: string) => {
  const iconMap: Record<string, any> = {
    info: InformationCircleOutline,
    warning: WarningOutline,
    error: CloseCircleOutline,
    success: CheckmarkCircleOutline
  }
  return iconMap[type] || InformationCircleOutline
}
</script>

<style scoped>
.log-container {
  padding: 8px;
}
.n-timeline-item {
  margin-bottom: 12px;
}
</style>
