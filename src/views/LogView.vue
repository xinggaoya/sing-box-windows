<template>
  <n-space vertical>
    <n-card>
      <n-space align="center" justify="space-between">
        <n-space align="center">
          <n-h3 style="margin: 0">运行日志</n-h3>
          <n-text depth="3">{{ logs.length }} 条记录</n-text>
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
            :disabled="!logs.length"
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
            :disabled="!logs.length"
          >
            <template #icon>
              <n-icon><CopyOutline /></n-icon>
            </template>
          </n-button>
        </n-space>
      </n-space>
    </n-card>

    <n-card>
      <n-scrollbar style="max-height: calc(100vh - 180px)" trigger="none">
        <n-infinite-scroll @load="handleLoad">
          <n-space vertical>
            <n-timeline>
              <n-timeline-item
                v-for="(log, index) in filteredLogs"
                :key="index"
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
              v-if="!filteredLogs.length"
              description="暂无日志记录"
            />
          </n-space>
        </n-infinite-scroll>
      </n-scrollbar>
    </n-card>
  </n-space>
</template>

<script setup lang="ts">
import { useInfoStore } from '@/stores/infoStore'
import { onMounted, ref, computed } from 'vue'
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

const message = useMessage()
const infoStore = useInfoStore()
const logs = ref<Log[]>([])
const filterType = ref<string | null>(null)

const logTypeOptions = [
  {
    label: '全部',
    value: null
  },
  {
    label: '信息',
    value: 'info'
  },
  {
    label: '警告',
    value: 'warning'
  },
  {
    label: '错误',
    value: 'error'
  },
  {
    label: '成功',
    value: 'success'
  }
]

const filteredLogs = computed(() => {
  if (!filterType.value) return logs.value
  return logs.value.filter(log => log.type === filterType.value)
})

onMounted(() => {
  loadInitialLogs()
})

const loadInitialLogs = () => {
  const initialLogs = infoStore.logs.slice(0, 20).map((log: any) => ({
    ...log,
    timestamp: Date.now()
  }))
  logs.value = initialLogs
}

const handleLoad = () => {
  const newLogs = infoStore.logs
    .slice(logs.value.length, logs.value.length + 20)
    .map((log: any) => ({
      ...log,
      timestamp: Date.now()
    }))
  logs.value.push(...newLogs)
}

const clearLogs = () => {
  logs.value = []
  message.success('日志已清空')
}

const copyLogs = () => {
  const logText = logs.value
    .map(log => `[${formatTime(log.timestamp)}] [${log.type}] ${log.payload}`)
    .join('\n')
  navigator.clipboard.writeText(logText)
  message.success('日志已复制到剪贴板')
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
.n-timeline-item {
  margin-bottom: 12px;
}
</style>
