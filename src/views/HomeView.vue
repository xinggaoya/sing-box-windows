<template>
  <div class="home-container">
    <!-- 状态控制卡片 -->
    <n-card class="control-card" :bordered="false">
      <n-space vertical :size="16">
        <!-- 状态指示器和控制按钮 -->
        <n-space justify="space-between" align="center">
          <n-space align="center" :size="16">
            <div class="status-indicator">
              <div class="status-dot" :class="{ active: appState.isRunning }"></div>
              <span class="status-text">{{ appState.isRunning ? '运行中' : '已停止' }}</span>
            </div>
            <n-tag
              :bordered="false"
              :type="appState.proxyMode === 'system' ? 'info' : 'warning'"
              size="medium"
              class="mode-tag"
            >
              <template #icon>
                <n-icon size="16">
                  <globe-outline v-if="appState.proxyMode === 'system'" />
                  <flash-outline v-else />
                </n-icon>
              </template>
              {{ appState.proxyMode === 'system' ? '系统代理' : 'TUN 模式' }}
            </n-tag>
          </n-space>
          <n-space :size="16">
            <n-button
              secondary
              type="info"
              size="medium"
              :disabled="!appState.isRunning"
              @click="onModeChange(appState.proxyMode === 'system' ? 'tun' : 'system')"
              class="control-button"
            >
              <template #icon>
                <n-icon><repeat-outline /></n-icon>
              </template>
              切换模式
            </n-button>
            <n-button
              secondary
              :type="appState.isRunning ? 'error' : 'primary'"
              size="medium"
              :loading="isStarting || isStopping"
              @click="appState.isRunning ? stopKernel() : runKernel()"
              class="control-button"
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
            <div class="traffic-icon-container">
              <n-icon size="22"><arrow-up-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">上传速度</span>
              <span class="traffic-value">{{ trafficStr.up }}</span>
            </div>
          </div>
          <div class="traffic-card download">
            <div class="traffic-icon-container">
              <n-icon size="22"><arrow-down-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">下载速度</span>
              <span class="traffic-value">{{ trafficStr.down }}</span>
            </div>
          </div>
          <div class="traffic-card upload-total">
            <div class="traffic-icon-container">
              <n-icon size="22"><cloud-upload-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">上传总流量</span>
              <span class="traffic-value">{{ uploadTotalTraffic }}</span>
            </div>
          </div>
          <div class="traffic-card download-total">
            <div class="traffic-icon-container">
              <n-icon size="22"><cloud-download-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">下载总流量</span>
              <span class="traffic-value">{{ downloadTotalTraffic }}</span>
            </div>
          </div>
          <div class="traffic-card memory">
            <div class="traffic-icon-container">
              <n-icon size="22"><hardware-chip-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">内存占用</span>
              <span class="traffic-value">{{ memoryStr }}</span>
            </div>
          </div>
          <div class="traffic-card active-connections">
            <div class="traffic-icon-container">
              <n-icon size="22"><git-network-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">活动连接</span>
              <span class="traffic-value">{{ activeConnectionsCount }}</span>
            </div>
          </div>
        </div>

        <!-- 流量图表 -->
        <div class="chart-wrapper">
          <TrafficChart
            :upload-speed="infoStore.traffic.up"
            :download-speed="infoStore.traffic.down"
          />
        </div>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { useMessage, useDialog } from 'naive-ui'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { formatBandwidth } from '@/utils'
import { Window } from '@tauri-apps/api/window'
import {
  PowerOutline,
  RepeatOutline,
  ArrowUpOutline,
  ArrowDownOutline,
  HardwareChipOutline,
  AnalyticsOutline,
  GlobeOutline,
  FlashOutline,
  CloudUploadOutline,
  CloudDownloadOutline,
  TimeOutline,
  GitNetworkOutline,
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores/AppStore'
import TrafficChart from '@/components/layout/TrafficChart.vue'
import { useInfoStore } from '@/stores/infoStore'
import { ProxyService } from '@/services/proxy-service'

const message = useMessage()
const dialog = useDialog()
const appState = useAppStore()
const infoStore = useInfoStore()
const proxyService = ProxyService.getInstance()
const isStarting = ref(false)
const isStopping = ref(false)

// 监听路由可见性变化，简化为只用于计算属性的控制
const route = useRoute()
const isRouteActive = computed(() => route.path === '/')

// 保留计算属性的可见性检查，但简化逻辑
const useTotalTraffic = computed(() => {
  if (!isRouteActive.value) return '0 B' // 不在当前路由时不计算
  return formatBandwidth(infoStore.traffic.total)
})

const memoryStr = computed(() => {
  if (!isRouteActive.value) return '0 B' // 不在当前路由时不计算
  return formatBandwidth(infoStore.memory.inuse)
})

const trafficStr = computed(() => {
  if (!isRouteActive.value) return { up: '0 B/s', down: '0 B/s' } // 不在当前路由时不计算
  return {
    up: formatBandwidth(Number(infoStore.traffic.up) || 0),
    down: formatBandwidth(Number(infoStore.traffic.down) || 0),
  }
})

const uploadTotalTraffic = computed(() => {
  if (!isRouteActive.value) return '0 B' // 不在当前路由时不计算
  return formatBandwidth(Number(infoStore.connectionsTotal.upload) || 0)
})

const downloadTotalTraffic = computed(() => {
  if (!isRouteActive.value) return '0 B' // 不在当前路由时不计算
  return formatBandwidth(Number(infoStore.connectionsTotal.download) || 0)
})

const activeConnectionsCount = computed(() => {
  if (!isRouteActive.value) return '0'
  return infoStore.connections.length.toString()
})

const formattedUptime = computed(() => {
  if (!isRouteActive.value) return '00:00:00' // 不在当前路由时不计算

  const uptime = Number(infoStore.uptime) || 0
  const hours = Math.floor(uptime / 3600)
  const minutes = Math.floor((uptime % 3600) / 60)
  const seconds = Math.floor(uptime % 60)
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
})

const runKernel = async () => {
  try {
    isStarting.value = true
    await infoStore.startKernel()
    appState.setRunningState(true)
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
    appState.setRunningState(false)
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
  padding: 12px 8px;
}

.control-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
}

.control-card :deep(.n-card__content) {
  padding: 16px;
}

.control-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: var(--n-text-color-disabled);
  transition: all 0.3s ease;
  position: relative;
}

.status-dot.active {
  background-color: var(--success-color);
  box-shadow: 0 0 8px var(--success-color);
}

.status-dot.active::after {
  content: '';
  position: absolute;
  top: -4px;
  left: -4px;
  right: -4px;
  bottom: -4px;
  border-radius: 50%;
  border: 1px solid var(--success-color);
  opacity: 0.4;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% {
    transform: scale(0.95);
    opacity: 0.6;
  }
  70% {
    transform: scale(1.1);
    opacity: 0.2;
  }
  100% {
    transform: scale(0.95);
    opacity: 0.6;
  }
}

.status-text {
  font-size: 15px;
  font-weight: 500;
  color: var(--n-text-color-1);
}

.mode-tag {
  padding: 0 12px;
  height: 30px;
  display: flex;
  align-items: center;
  gap: 6px;
  border-radius: 8px;
  font-weight: 500;
}

.traffic-monitor {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
  margin: 0;
}

.traffic-card {
  padding: 14px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  gap: 14px;
  transition: all 0.3s ease;
  border: 1px solid var(--n-border-color);
}

.traffic-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.05);
}

:deep(.dark) .traffic-card:hover {
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
}

.traffic-icon-container {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 42px;
  height: 42px;
  border-radius: 12px;
  flex-shrink: 0;
}

.upload .traffic-icon-container {
  background-color: rgba(24, 160, 88, 0.08);
  color: var(--success-color);
}

.download .traffic-icon-container {
  background-color: rgba(32, 128, 240, 0.08);
  color: var(--primary-color);
}

.memory .traffic-icon-container {
  background-color: rgba(208, 48, 80, 0.08);
  color: var(--error-color);
}

.total .traffic-icon-container {
  background-color: rgba(240, 160, 32, 0.08);
  color: var(--warning-color);
}

.upload-total .traffic-icon-container {
  background-color: rgba(60, 180, 100, 0.08);
  color: #2a9d8f;
}

.download-total .traffic-icon-container {
  background-color: rgba(80, 140, 220, 0.08);
  color: #4c6ef5;
}

.uptime .traffic-icon-container {
  background-color: rgba(160, 100, 200, 0.08);
  color: #9d4edd;
}

.active-connections .traffic-icon-container {
  background-color: rgba(100, 160, 200, 0.08);
  color: #3598db;
}

:deep(.dark) .upload .traffic-icon-container {
  background-color: rgba(24, 160, 88, 0.15);
}

:deep(.dark) .download .traffic-icon-container {
  background-color: rgba(32, 128, 240, 0.15);
}

:deep(.dark) .memory .traffic-icon-container {
  background-color: rgba(208, 48, 80, 0.15);
}

:deep(.dark) .total .traffic-icon-container {
  background-color: rgba(240, 160, 32, 0.15);
}

:deep(.dark) .upload-total .traffic-icon-container {
  background-color: rgba(60, 180, 100, 0.15);
}

:deep(.dark) .download-total .traffic-icon-container {
  background-color: rgba(80, 140, 220, 0.15);
}

:deep(.dark) .uptime .traffic-icon-container {
  background-color: rgba(160, 100, 200, 0.15);
}

:deep(.dark) .active-connections .traffic-icon-container {
  background-color: rgba(100, 160, 200, 0.15);
}

.traffic-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.traffic-label {
  font-size: 13px;
  color: var(--n-text-color-2);
}

.traffic-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.chart-wrapper {
  margin-top: 6px;
  height: 260px;
  border-radius: 14px;
  overflow: hidden;
  border: 1px solid rgba(0, 0, 0, 0.05);
}

.control-button {
  border-radius: 10px;
  font-weight: 500;
  transition: all 0.25s ease;
}

.control-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
</style>
