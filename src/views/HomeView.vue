<template>
  <div class="home-container">
    <!-- 状态控制卡片 -->
    <n-card class="control-card" :bordered="false">
      <n-space vertical :size="24">
        <!-- 状态指示器和控制按钮 -->
        <n-space justify="space-between" align="center">
          <n-space align="center" :size="16">
            <div class="status-indicator">
              <div class="status-dot" :class="{ active: appState.isRunning }" />
              <span class="status-text">{{ appState.isRunning ? '运行中' : '已停止' }}</span>
            </div>
            <n-tag :bordered="false" type="info" size="small">
              {{ appState.mode === 'system' ? '系统代理' : 'TUN 模式' }}
            </n-tag>
          </n-space>
          <n-space :size="12">
            <n-button
              secondary
              type="info"
              size="small"
              :disabled="!appState.isRunning"
              @click="onModeChange(appState.mode === 'system' ? 'tun' : 'system')"
            >
              <template #icon>
                <n-icon><repeat-outline /></n-icon>
              </template>
              切换模式
            </n-button>
            <n-button
              secondary
              :type="appState.isRunning ? 'error' : 'primary'"
              size="small"
              :loading="isStarting || isStopping"
              @click="appState.isRunning ? stopKernel() : runKernel()"
            >
              <template #icon>
                <n-icon>
                  <power-outline />
                </n-icon>
              </template>
              {{ appState.isRunning ? '停止' : '启动' }}
            </n-button>
          </n-space>
        </n-space>

        <!-- 实时流量监控 -->
        <div class="traffic-monitor">
          <div class="traffic-card upload">
            <n-icon size="22" color="#18a058"><arrow-up-outline /></n-icon>
            <div class="traffic-info">
              <span class="traffic-label">上传</span>
              <span class="traffic-value">{{ trafficStr.up }}</span>
            </div>
          </div>
          <div class="traffic-card download">
            <n-icon size="22" color="#2080f0"><arrow-down-outline /></n-icon>
            <div class="traffic-info">
              <span class="traffic-label">下载</span>
              <span class="traffic-value">{{ trafficStr.down }}</span>
            </div>
          </div>
          <div class="traffic-card memory">
            <n-icon size="22" color="#d03050"><hardware-chip-outline /></n-icon>
            <div class="traffic-info">
              <span class="traffic-label">内存</span>
              <span class="traffic-value">{{ memoryStr }}</span>
            </div>
          </div>
          <div class="traffic-card total">
            <n-icon size="22" color="#f0a020"><analytics-outline /></n-icon>
            <div class="traffic-info">
              <span class="traffic-label">总流量</span>
              <span class="traffic-value">{{ useTotalTraffic }}</span>
            </div>
          </div>
        </div>

        <!-- 流量图表 -->
        <div class="chart-wrapper">
          <Echarts
            :download-speed="infoStore.traffic.up"
            :upload-speed="infoStore.traffic.down"
            :is-visible="isWindowVisible"
          />
        </div>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { useMessage, useDialog } from 'naive-ui'
import { computed, ref, onUnmounted, onMounted } from 'vue'
import { formatBandwidth } from '@/utils'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'
import {
  PowerOutline,
  RepeatOutline,
  ArrowUpOutline,
  ArrowDownOutline,
  HardwareChipOutline,
  AnalyticsOutline,
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores/AppStore'
import Echarts from '@/components/layout/Echarts.vue'
import { useInfoStore } from '@/stores/infoStore'
import { ProxyService } from '@/services/proxy-service'

const message = useMessage()
const dialog = useDialog()
const appState = useAppStore()
const infoStore = useInfoStore()
const proxyService = ProxyService.getInstance()
const isStarting = ref(false)
const isStopping = ref(false)
const isWindowVisible = ref(true)

// 监听窗口事件
const setupWindowListeners = () => {
  mitt.on('window-minimize', () => {
    isWindowVisible.value = false
  })

  mitt.on('window-hide', () => {
    isWindowVisible.value = false
  })

  mitt.on('window-show', () => {
    isWindowVisible.value = true
  })

  mitt.on('window-restore', () => {
    isWindowVisible.value = true
  })
}

onMounted(() => {
  setupWindowListeners()
})

onUnmounted(() => {
  mitt.off('window-minimize')
  mitt.off('window-hide')
  mitt.off('window-show')
  mitt.off('window-restore')
})

const useTotalTraffic = computed(() => {
  return formatBandwidth(infoStore.traffic.total)
})

const memoryStr = computed(() => formatBandwidth(infoStore.memory.inuse))

const trafficStr = computed(() => ({
  up: formatBandwidth(Number(infoStore.traffic.up) || 0),
  down: formatBandwidth(Number(infoStore.traffic.down) || 0),
}))

const runKernel = async () => {
  try {
    isStarting.value = true
    await infoStore.startKernel()
    appState.isRunning = true
    message.success('内核已启动')
  } catch (error) {
    message.error(error as string)
  } finally {
    isStarting.value = false
  }
}

const stopKernel = async () => {
  try {
    isStopping.value = true
    await infoStore.stopKernel()
    appState.isRunning = false
    message.success('内核已停止')
  } catch (error) {
    message.error(error as string)
  } finally {
    isStopping.value = false
  }
}

const onModeChange = async (value: string) => {
  const showMessage = (type: 'success' | 'info' | 'error', content: string) => {
    switch (type) {
      case 'success':
        message.success(content)
        break
      case 'info':
        message.info(content)
        break
      case 'error':
        message.error(content)
        break
    }
  }

  const needClose = await proxyService.switchMode(value as 'system' | 'tun', showMessage)
  if (needClose) {
    const appWindow = Window.getCurrent()
    await appWindow.close()
  }
}
</script>

<style scoped>
.home-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px;
}

.control-card {
  border-radius: 16px;
  transition: all 0.3s ease;
}

.control-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--n-text-color-disabled);
  transition: all 0.3s ease;
}

.status-dot.active {
  background-color: var(--success-color);
  box-shadow: 0 0 8px var(--success-color);
}

.status-text {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-color-1);
}

.traffic-monitor {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin: 0 -8px;
}

.traffic-card {
  background-color: var(--card-color);
  border-radius: 12px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  transition: all 0.3s ease;
  cursor: default;
}

.traffic-card:hover {
  transform: translateY(-2px);
  background-color: var(--card-color-hover);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.traffic-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.traffic-label {
  font-size: 13px;
  color: var(--text-color-2);
}

.traffic-value {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-color-1);
}

.chart-wrapper {
  margin-top: 8px;
  height: calc(100vh - 380px);
  min-height: 300px;
  border-radius: 12px;
  overflow: hidden;
  background-color: var(--card-color);
  transition: all 0.3s ease;
}

.chart-wrapper:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

:deep(.n-button) {
  font-weight: 500;
}

:deep(.n-tag) {
  font-weight: 500;
}
</style>
