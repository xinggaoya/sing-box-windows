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
            {{ t('log.title') }}
          </n-h3>
          <n-tag :bordered="false" type="info" size="medium" class="log-count-tag">
            {{ displayedLogs.length }}/{{ totalLogs }} {{ t('log.records') }}
          </n-tag>
        </div>
        <n-space :size="12">
          <n-switch v-model:value="autoScroll" size="medium" class="auto-scroll-switch">
            <template #checked>{{ t('log.autoScroll') }}</template>
            <template #unchecked>{{ t('log.manualScroll') }}</template>
          </n-switch>

          <n-select
            v-model:value="filterType"
            :options="logTypeOptions"
            :placeholder="t('log.filterType')"
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
              {{ t('log.clear') }}
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
              {{ t('log.copy') }}
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
              {{ t('log.export') }}
            </n-tooltip>
          </n-space>
        </n-space>
      </n-space>
    </n-card>

    <!-- 搜索和统计卡片 -->
    <n-card class="log-search-card" :bordered="false">
      <div class="search-stats-container">
        <div class="search-container">
          <n-input 
            v-model:value="searchQuery" 
            :placeholder="t('log.searchLogs')" 
            clearable
            class="search-input"
          >
            <template #prefix>
              <n-icon><search-outline /></n-icon>
            </template>
          </n-input>
        </div>
        <div class="stats-container">
          <n-space justify="end" align="center" :wrap="false">
            <n-statistic v-for="(count, type) in logTypeCounts" :key="type" class="log-type-statistic">
              <template #label>
                <n-tag :type="getLogTagType(type)" size="small" round>
                  {{ t(`log.types.${type}`) }}
                </n-tag>
              </template>
              <template #prefix>
                <n-icon :color="getLogColor(type)">
                  <information-circle-outline v-if="type === 'info'" />
                  <warning-outline v-else-if="type === 'warning'" />
                  <alert-circle-outline v-else-if="type === 'error'" />
                  <checkmark-circle-outline v-else-if="type === 'success'" />
                </n-icon>
              </template>
              {{ count }}
            </n-statistic>
          </n-space>
        </div>
      </div>
    </n-card>

    <!-- 日志内容卡片 -->
    <n-card class="log-content-card" :bordered="false">
      <div class="log-content-wrapper">
        <div v-if="displayedLogs.length">
          <!-- 使用 Naive UI 的虚拟列表组件 -->
          <n-virtual-list
            ref="virtualListRef"
            class="log-virtual-list"
            :items="formattedLogs"
            :item-size="68"
            :show-scrollbar="true"
            container-style="max-height: calc(100vh - 320px); overflow: auto;"
            @scroll="handleVirtualScroll"
          >
            <template #default="{ item }">
              <div class="log-item" :key="item.key" :class="`log-item-${item.type}`">
                <div class="log-item-header">
                  <n-tag :type="getLogTagType(item.type)" size="small" round class="log-tag">
                    {{ t(`log.types.${item.type}`) }}
                  </n-tag>
                  <span class="log-time">{{ formatTime(item.timestamp, true) }}</span>
                </div>
                <div class="log-message-container">
                  <span class="log-message" :class="getLogClass(item.type)">
                    <n-ellipsis v-if="item.payload.length > 150" :line-clamp="2" expand-trigger="click">
                      <template v-if="searchQuery && item.payload.toLowerCase().includes(searchQuery.toLowerCase())">
                        <highlight-text :text="item.payload" :keyword="searchQuery" />
                      </template>
                      <template v-else>
                        {{ item.payload }}
                      </template>
                    </n-ellipsis>
                    <template v-else>
                      <template v-if="searchQuery && item.payload.toLowerCase().includes(searchQuery.toLowerCase())">
                        <highlight-text :text="item.payload" :keyword="searchQuery" />
                      </template>
                      <template v-else>
                        {{ item.payload }}
                      </template>
                    </template>
                  </span>
                </div>
              </div>
            </template>
          </n-virtual-list>
        </div>
        <n-empty v-else-if="searchActive && !displayedLogs.length" :description="t('log.noSearchResults')" class="log-empty" />
        <n-empty v-else :description="t('log.noLogs')" class="log-empty" />
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
  CheckmarkCircleOutline
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
      required: true
    },
    keyword: {
      type: String,
      required: true
    }
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
          h('span', { 
            style: { 
              backgroundColor: 'rgba(var(--primary-color), 0.1)',
              fontWeight: 'bold',
              padding: '0 2px',
              borderRadius: '2px'
            } 
          }, text.substring(index, index + keyword.length))
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
  }
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
    success: 0
  }
  
  logStore.logs.forEach(log => {
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
      log.payload.toLowerCase().includes(searchQuery.value.toLowerCase())
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
watch([filterType, searchQuery], () => {
  updateDisplayedLogs()
  nextTick(() => {
    if (autoScroll.value) {
      scrollToBottom()
    }
  })
}, { immediate: true })

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

.log-header-card,
.log-search-card,
.log-content-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.log-header-card:hover,
.log-search-card:hover,
.log-content-card:hover {
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

.search-stats-container {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
}

.search-container {
  flex: 1;
  min-width: 250px;
}

.search-input {
  width: 100%;
  max-width: 400px;
  transition: all 0.3s ease;
}

.search-input:hover {
  transform: translateY(-1px);
}

.stats-container {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  justify-content: flex-end;
}

.log-type-statistic {
  margin-left: 14px;
  min-width: 60px;
  text-align: center;
}

.log-content-card {
  flex: 1;
  border-radius: 16px;
  min-height: 300px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.log-content-wrapper {
  height: 100%;
  padding: 8px 4px;
  position: relative;
}

.log-virtual-list {
  height: calc(100vh - 320px);
  padding: 4px;
}

.log-item {
  padding: 10px 14px;
  border-radius: 8px;
  margin-bottom: 8px;
  transition: all 0.3s ease;
  border-left: 3px solid transparent;
  display: flex;
  flex-direction: column;
  gap: 6px;
  position: relative;
}

.log-item:hover {
  transform: translateX(2px);
  background-color: rgba(var(--primary-color), 0.03);
}

.log-item-header {
  display: flex;
  align-items: center;
  gap: 10px;
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

.log-message-container {
  padding-left: 8px;
  border-left: 1px dashed rgba(var(--primary-color), 0.2);
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

.log-item-info {
  border-left-color: var(--info-color);
}

.log-item-warning {
  border-left-color: var(--warning-color);
}

.log-item-error {
  border-left-color: var(--error-color);
}

.log-item-success {
  border-left-color: var(--success-color);
}

@media (max-width: 768px) {
  .search-stats-container {
    flex-direction: column;
    align-items: stretch;
  }
  
  .stats-container {
    justify-content: space-between;
  }
  
  .log-type-statistic {
    margin-left: 0;
  }
}
</style>
