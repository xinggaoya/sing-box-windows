<template>
  <div class="home-container">
    <!-- 状态控制卡片 -->
    <n-card class="control-card" :bordered="false">
      <n-space vertical :size="16">
        <!-- 状态指示器和控制按钮 -->
        <n-space justify="space-between" align="center" wrap-item>
          <div class="status-indicator">
            <div class="status-dot" :class="{ active: appState.isRunning }"></div>
            <span class="status-text">{{
              appState.isRunning ? t('home.status.running') : t('home.status.stopped')
            }}</span>
            <n-tag :bordered="false" :type="appState.wsConnected ? 'success' : 'error'" size="medium" class="ws-tag">
              <template #icon>
                <n-icon size="16">
                  <wifi-outline v-if="appState.wsConnected" />
                  <close-circle-outline v-else />
                </n-icon>
              </template>
              {{ appState.wsConnected ? t('home.wsStatus.connected') : t('home.wsStatus.disconnected') }}
            </n-tag>
            <n-tag :bordered="false" :type="isAdmin ? 'success' : 'warning'" size="medium" class="admin-tag">
              <template #icon>
                <n-icon size="16">
                  <shield-checkmark-outline v-if="isAdmin" />
                  <shield-outline v-else />
                </n-icon>
              </template>
              {{ isAdmin ? t('home.adminStatus.admin') : t('home.adminStatus.normal') }}
            </n-tag>
          </div>
          <div class="controls-wrapper">

            <!-- 管理员重启按钮 - 仅非管理员状态显示 -->
            <n-button v-if="!isAdmin" type="warning" secondary size="medium" @click="restartAsAdmin"
              class="control-button">
              <template #icon>
                <n-icon>
                  <shield-checkmark-outline />
                </n-icon>
              </template>
              {{ t('home.restartAsAdmin') }}
            </n-button>
            <!-- 启动/停止按钮 -->
            <n-button :type="appState.isRunning ? 'error' : 'primary'" size="medium" :loading="isStarting || isStopping"
              @click="appState.isRunning ? stopKernel() : runKernel()" class="control-button">
              <template #icon>
                <n-icon>
                  <power-outline />
                </n-icon>
              </template>
              {{ appState.isRunning ? t('home.stop') : t('home.start') }}
            </n-button>
          </div>
        </n-space>

        <!-- 状态标签 -->
        <n-space :size="12" class="status-tags">
          <div>
            <!-- 代理模式选择器 -->
            <n-radio-group v-model:value="currentProxyMode" name="proxy-mode"
              :disabled="isSwitching || isStarting || isStopping" class="proxy-mode-selector">
              <n-radio-button v-for="mode in proxyModes" :key="mode.value" :value="mode.value"
                :disabled="mode.value === 'tun' && !isAdmin">
                <n-tooltip placement="top" trigger="hover">
                  <template #trigger>
                    <n-space :size="4" align="center">
                      <n-icon>
                        <component :is="mode.icon" />
                      </n-icon>
                      <span>{{ t(mode.nameKey) }}</span>
                    </n-space>
                  </template>
                  {{ t(mode.tipKey) }}
                </n-tooltip>
              </n-radio-button>
            </n-radio-group>
          </div>
        </n-space>

        <!-- 实时流量监控 -->
        <div class="traffic-monitor">
          <div class="traffic-card upload">
            <div class="traffic-icon-container">
              <n-icon size="22"><arrow-up-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">{{ t('home.traffic.uploadSpeed') }}</span>
              <span class="traffic-value">{{ trafficStr.up }}</span>
            </div>
          </div>
          <div class="traffic-card download">
            <div class="traffic-icon-container">
              <n-icon size="22"><arrow-down-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">{{ t('home.traffic.downloadSpeed') }}</span>
              <span class="traffic-value">{{ trafficStr.down }}</span>
            </div>
          </div>
          <div class="traffic-card upload-total">
            <div class="traffic-icon-container">
              <n-icon size="22"><cloud-upload-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">{{ t('home.traffic.uploadTotal') }}</span>
              <span class="traffic-value">{{ uploadTotalTraffic }}</span>
            </div>
          </div>
          <div class="traffic-card download-total">
            <div class="traffic-icon-container">
              <n-icon size="22"><cloud-download-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">{{ t('home.traffic.downloadTotal') }}</span>
              <span class="traffic-value">{{ downloadTotalTraffic }}</span>
            </div>
          </div>
          <div class="traffic-card memory">
            <div class="traffic-icon-container">
              <n-icon size="22"><hardware-chip-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">{{ t('home.traffic.memory') }}</span>
              <span class="traffic-value">{{ memoryStr }}</span>
            </div>
          </div>
          <div class="traffic-card active-connections">
            <div class="traffic-icon-container">
              <n-icon size="22"><git-network-outline /></n-icon>
            </div>
            <div class="traffic-info">
              <span class="traffic-label">{{ t('home.traffic.connections') }}</span>
              <span class="traffic-value">{{ activeConnectionsCount }}</span>
            </div>
          </div>
        </div>

        <!-- 流量图表 -->
        <div class="chart-wrapper">
          <TrafficChart :upload-speed="trafficStore.traffic.up" :download-speed="trafficStore.traffic.down" />
        </div>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { useMessage, useDialog } from 'naive-ui'
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
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
  ShieldCheckmarkOutline,
  ShieldOutline,
  WifiOutline,
  CloseCircleOutline,
  SettingsOutline
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores/app/AppStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import TrafficChart from '@/components/layout/TrafficChart.vue'
import { ProxyService } from '@/services/proxy-service'
import { useI18n } from 'vue-i18n'
import { tauriApi } from '@/services/tauri-api'

const message = useMessage()
const dialog = useDialog()
const appState = useAppStore()
const kernelStore = useKernelStore()
const trafficStore = useTrafficStore()
const connectionStore = useConnectionStore()
const proxyService = ProxyService.getInstance()
const isStarting = ref(false)
const isStopping = ref(false)
const isSwitching = ref(false)
const { t } = useI18n()
const isAdmin = ref(false)

// 代理模式选择
const currentProxyMode = ref(appState.proxyMode || 'system')

// 定义代理模式数据
const proxyModes = [
  {
    value: 'system',
    nameKey: 'home.proxyMode.system',
    tipKey: 'home.proxyMode.systemTip',
    icon: GlobeOutline
  },
  {
    value: 'manual',
    nameKey: 'home.proxyMode.manual',
    tipKey: 'home.proxyMode.manualTip',
    icon: SettingsOutline
  },
  {
    value: 'tun',
    nameKey: 'home.proxyMode.tun',
    tipKey: 'home.proxyMode.tunTip',
    icon: FlashOutline
  }
]

// 监听代理模式变化
watch(currentProxyMode, async (newMode) => {
  if (newMode !== appState.proxyMode) {
    await onModeChange(newMode)
  }
})

// 监听路由可见性变化，简化为只用于计算属性的控制
const route = useRoute()
const isRouteActive = computed(() => route.path === '/')

// 添加加载状态
const isTrafficLoading = ref(false)
const isConnectionLoading = ref(false)

// 保留计算属性的可见性检查，但简化逻辑
const useTotalTraffic = computed(() => {
  if (!isRouteActive.value) return '0 B' // 不在当前路由时不计算
  return formatBandwidth(trafficStore.traffic.total)
})

const memoryStr = computed(() => {
  if (!isRouteActive.value) return '0 B' // 不在当前路由时不计算
  return formatBandwidth(connectionStore.memory?.inuse || 0)
})

const trafficStr = computed(() => {
  if (!isRouteActive.value) return { up: '0 B/s', down: '0 B/s' } // 不在当前路由时不计算
  return {
    up: formatBandwidth(Number(trafficStore.traffic.up) || 0),
    down: formatBandwidth(Number(trafficStore.traffic.down) || 0),
  }
})

const uploadTotalTraffic = computed(() => {
  if (!isRouteActive.value) return '0 B' // 不在当前路由时不计算
  return formatBandwidth(Number(trafficStore.traffic.totalUp) || 0)
})

const downloadTotalTraffic = computed(() => {
  if (!isRouteActive.value) return '0 B' // 不在当前路由时不计算
  return formatBandwidth(Number(trafficStore.traffic.totalDown) || 0)
})

const activeConnectionsCount = computed(() => {
  if (!isRouteActive.value) return '0'
  return connectionStore.connections.length.toString()
})

const formattedUptime = computed(() => {
  if (!isRouteActive.value) return '00:00:00' // 不在当前路由时不计算

  const uptime = Number(kernelStore.uptime) || 0
  const hours = Math.floor(uptime / 3600)
  const minutes = Math.floor((uptime % 3600) / 60)
  const seconds = Math.floor(uptime % 60)
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
})

const runKernel = async () => {
  try {
    isStarting.value = true
    await kernelStore.startKernel()
    appState.setRunningState(true)
    message.success(t('notification.kernelStarted'))
  } catch (error) {
    message.error(error as string)
  } finally {
    isStarting.value = false
  }
}

const stopKernel = async () => {
  try {
    isStopping.value = true
    await kernelStore.stopKernel()
    appState.setRunningState(false)
    message.success(t('notification.kernelStopped'))
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

  try {
    isSwitching.value = true

    // 如果内核正在运行，需要重启
    if (appState.isRunning) {
      // 先停止内核
      await kernelStore.stopKernel()
      appState.setRunningState(false)
    }

    // 切换模式
    let needClose = false;

    if (value === 'system') {
      await tauriApi.proxy.setSystemProxy();
      appState.setProxyMode('system');
      showMessage('success', t('notification.systemProxyEnabled'));
    } else if (value === 'manual') {
      await tauriApi.proxy.setManualProxy();
      appState.setProxyMode('manual');
      showMessage('info', t('notification.manualProxyEnabled'));
    } else if (value === 'tun') {
      needClose = await proxyService.switchMode('tun', showMessage);
    }

    // 如果内核之前在运行，重新启动
    if (appState.isRunning) {
      await kernelStore.startKernel()
      appState.setRunningState(true)
      message.success(t('notification.kernelRestarted'))
    }

    if (needClose) {
      const appWindow = Window.getCurrent()
      await appWindow.close()
    }
  } catch (error) {
    message.error(error as string)
  } finally {
    isSwitching.value = false
  }
}

// 检查管理员权限
const checkAdminStatus = async () => {
  try {
    isAdmin.value = await tauriApi.system.checkAdmin()
  } catch (error) {
    console.error('检查管理员权限失败:', error)
  }
}

// 在路由可见时重新设置监听
const setupListeners = async () => {
  try {
    if (appState.isRunning) {
      console.log("HomeView: 尝试设置监听器")

      // 清理之前的监听器，确保没有重复监听
      trafficStore.cleanupListeners()
      connectionStore.cleanupListeners()

      // 设置监听器，添加等待确保setup完成
      isTrafficLoading.value = true
      isConnectionLoading.value = true

      // 使用Promise.all同时设置两个监听器
      await Promise.all([
        trafficStore.setupTrafficListener(),
        connectionStore.setupConnectionsListener(),
        connectionStore.setupMemoryListener()
      ]).catch(e => {
        console.error("HomeView: 设置监听器失败", e)
        // 尝试重试一次
        return new Promise(resolve => {
          setTimeout(async () => {
            try {
              await trafficStore.setupTrafficListener()
              await connectionStore.setupConnectionsListener()
              await connectionStore.setupMemoryListener()
              resolve(true)
            } catch (retryError) {
              console.error("HomeView: 重试设置监听器失败", retryError)
              resolve(false)
            }
          }, 1000)
        })
      })

      isTrafficLoading.value = false
      isConnectionLoading.value = false
      console.log("HomeView: 监听器设置完成")
    }
  } catch (error) {
    console.error('HomeView: 设置监听器失败:', error)
    isTrafficLoading.value = false
    isConnectionLoading.value = false
  }
}

const restartAsAdmin = async () => {
  try {
    await tauriApi.system.restartAsAdmin()
    message.info(t('notification.restartingAsAdmin') || '正在以管理员身份重启应用...')
    // 重启后应用会关闭，所以不需要处理成功回调
  } catch (error) {
    message.error(error as string)
  }
}

onMounted(async () => {
  // 更新当前代理模式
  currentProxyMode.value = appState.proxyMode;

  // 设置监听器
  await setupListeners()

  // 检查管理员权限
  await checkAdminStatus()

  // 监听路由变化，当返回到主页时重新设置监听器
  watch(isRouteActive, (isActive) => {
    if (isActive && appState.isRunning) {
      setupListeners()
    } else if (!isActive) {
      // 不在当前页面时清理监听器，减少资源占用
      trafficStore.cleanupListeners()
      connectionStore.cleanupListeners()
    }
  })

  // 监听内核状态变化
  watch(() => appState.isRunning, (isRunning) => {
    if (isRunning && isRouteActive.value) {
      setupListeners()
    } else if (!isRunning) {
      // 内核停止时清理监听器
      trafficStore.cleanupListeners()
      connectionStore.cleanupListeners()
    }
  })
})

// 组件卸载时清理
onUnmounted(() => {
  // 清理流量监听器
  trafficStore.cleanupListeners()

  // 清理连接监听器
  connectionStore.cleanupListeners()
})

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

.controls-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.proxy-mode-selector {
  margin-right: 4px;
}

.mode-radio {
  display: flex;
  align-items: center;
}

.status-tags {
  margin-top: -8px;
}

.mode-tag,
.admin-tag,
.ws-tag {
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
