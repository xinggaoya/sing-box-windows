<template>
  <div class="home-container">
    <!-- 顶部状态卡片 -->
    <n-card class="status-card" :bordered="false">
      <div class="status-header">
        <div class="status-left">
          <div class="status-indicator">
            <div class="status-dot" :class="{ active: appState.isRunning }"></div>
            <span class="status-text">{{
              appState.isRunning ? t('home.status.running') : t('home.status.stopped')
            }}</span>
          </div>
          <div class="status-tags">
            <n-tag
              :bordered="false"
              :type="appState.wsConnected ? 'success' : 'error'"
              class="status-tag"
            >
              <template #icon>
                <n-icon size="16">
                  <wifi-outline v-if="appState.wsConnected" />
                  <close-circle-outline v-else />
                </n-icon>
              </template>
              {{
                appState.wsConnected
                  ? t('home.wsStatus.connected')
                  : t('home.wsStatus.disconnected')
              }}
            </n-tag>
            <n-tag :bordered="false" :type="isAdmin ? 'success' : 'warning'" class="status-tag">
              <template #icon>
                <n-icon size="16">
                  <shield-checkmark-outline v-if="isAdmin" />
                  <shield-outline v-else />
                </n-icon>
              </template>
              {{ isAdmin ? t('home.adminStatus.admin') : t('home.adminStatus.normal') }}
            </n-tag>
          </div>
        </div>
        <div class="status-right">
          <!-- 启动/停止按钮 -->
          <n-button
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
            {{ appState.isRunning ? t('home.stop') : t('home.start') }}
          </n-button>
        </div>
      </div>
    </n-card>

    <!-- 代理模式设置卡片 -->
    <div class="proxy-modes-container">
      <!-- 流量代理模式卡片 -->
      <n-card class="mode-card" :bordered="false">
        <div class="mode-card-header">
          <n-icon size="20" class="mode-card-icon">
            <layers-outline />
          </n-icon>
          <h3 class="mode-card-title">{{ t('home.proxyHeader.flowMode') }}</h3>
        </div>
        <div class="mode-card-content">
          <div class="mode-buttons">
            <n-button-group size="medium">
              <n-button 
                v-for="mode in proxyModes"
                :key="mode.value"
                :type="currentProxyMode === mode.value ? 'primary' : 'default'"
                :disabled="isSwitching || isStarting || isStopping"
                @click="onModeChange(mode.value)"
              >
                <template #icon>
                  <n-icon>
                    <component :is="mode.icon" />
                  </n-icon>
                </template>
                {{ t(mode.nameKey) }}
              </n-button>
            </n-button-group>
          </div>
          <div class="mode-description">
            {{ currentProxyMode ? t(`home.proxyMode.${currentProxyMode}Description`) : '' }}
          </div>
        </div>
      </n-card>

      <!-- 节点代理模式卡片 -->
      <n-card class="mode-card" :bordered="false">
        <div class="mode-card-header">
          <n-icon size="20" class="mode-card-icon">
            <git-network-outline />
          </n-icon>
          <h3 class="mode-card-title">{{ t('home.proxyHeader.nodeMode') }}</h3>
        </div>
        <div class="mode-card-content">
          <div class="mode-buttons">
            <n-button-group size="medium">
              <n-button
                v-for="mode in nodeProxyModes"
                :key="mode.value"
                :type="currentNodeProxyMode === mode.value ? 'primary' : 'default'"
                :disabled="!appState.isRunning || isSwitching || isStarting || isStopping"
                @click="handleNodeProxyModeChange(mode.value)"
              >
                <template #icon>
                  <n-icon>
                    <component :is="mode.icon" />
                  </n-icon>
                </template>
                {{ mode.label }}
              </n-button>
            </n-button-group>
          </div>
          <div class="mode-description">
            {{ currentNodeProxyMode ? t(`proxy.mode.${currentNodeProxyMode}Description`) : '' }}
          </div>
        </div>
      </n-card>
    </div>

    <!-- 节点模式切换确认对话框 -->
    <n-modal
      v-model:show="showNodeModeChangeModal"
      preset="dialog"
      :title="`${t('proxy.switchTo')}${targetNodeProxyMode ? getNodeProxyModeText(targetNodeProxyMode) : ''}`"
    >
      <template #header>
        <div class="modal-header">
          <n-icon size="22" class="modal-icon">
            <information-circle-outline />
          </n-icon>
          <span
            >{{ t('proxy.switchTo')
            }}{{ targetNodeProxyMode ? getNodeProxyModeText(targetNodeProxyMode) : '' }}</span
          >
        </div>
      </template>
      <div class="modal-content">{{ t('proxy.switchModeConfirm') }}</div>
      <template #action>
        <div class="modal-footer">
          <n-space justify="end">
            <n-button @click="showNodeModeChangeModal = false">{{ t('common.cancel') }}</n-button>
            <n-button
              type="primary"
              :loading="isChangingNodeMode"
              @click="confirmNodeProxyModeChange"
            >
              {{ t('proxy.confirmSwitch') }}
            </n-button>
          </n-space>
        </div>
      </template>
    </n-modal>

    <!-- 流量数据卡片 -->
    <n-card class="stats-card" :bordered="false">
      <template #header>
        <div class="stats-header">
          <h3 class="stats-title">
            <n-icon size="18" class="stats-icon">
              <analytics-outline />
            </n-icon>
            {{ t('home.traffic.title') }}
          </h3>
          <div class="connections-indicator">
            <n-icon size="16">
              <git-network-outline />
            </n-icon>
            <span>{{ activeConnectionsCount }} {{ t('home.traffic.connectionsLabel') }}</span>
          </div>
        </div>
      </template>

      <div class="traffic-content">
        <!-- 实时流量统计 -->
        <div class="traffic-stats">
          <div class="traffic-row">
            <div class="traffic-item">
              <div class="traffic-label">
                <n-icon size="16" class="traffic-icon upload-icon">
                  <arrow-up-outline />
                </n-icon>
                <span>{{ t('home.traffic.uploadSpeed') }}</span>
              </div>
              <div class="traffic-value">{{ trafficStr.up }}</div>
            </div>

            <div class="traffic-item">
              <div class="traffic-label">
                <n-icon size="16" class="traffic-icon download-icon">
                  <arrow-down-outline />
                </n-icon>
                <span>{{ t('home.traffic.downloadSpeed') }}</span>
              </div>
              <div class="traffic-value">{{ trafficStr.down }}</div>
            </div>

            <div class="traffic-item">
              <div class="traffic-label">
                <n-icon size="16" class="traffic-icon cloud-up-icon">
                  <cloud-upload-outline />
                </n-icon>
                <span>{{ t('home.traffic.uploadTotal') }}</span>
              </div>
              <div class="traffic-value">{{ uploadTotalTraffic }}</div>
            </div>

            <div class="traffic-item">
              <div class="traffic-label">
                <n-icon size="16" class="traffic-icon cloud-down-icon">
                  <cloud-download-outline />
                </n-icon>
                <span>{{ t('home.traffic.downloadTotal') }}</span>
              </div>
              <div class="traffic-value">{{ downloadTotalTraffic }}</div>
            </div>

            <div class="traffic-item">
              <div class="traffic-label">
                <n-icon size="16" class="traffic-icon memory-icon">
                  <hardware-chip-outline />
                </n-icon>
                <span>{{ t('home.traffic.memory') }}</span>
              </div>
              <div class="traffic-value">{{ memoryStr }}</div>
            </div>
          </div>
        </div>

        <!-- 流量图表 -->
        <div class="chart-container">
          <TrafficChart
            :upload-speed="trafficStore.traffic.up"
            :download-speed="trafficStore.traffic.down"
            class="traffic-chart"
          />
        </div>
      </div>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { useMessage, useDialog } from 'naive-ui'
import { computed, ref, onMounted, onUnmounted, watch, h } from 'vue'
import { useRoute } from 'vue-router'
import { formatBandwidth } from '@/utils'
import { Window } from '@tauri-apps/api/window'
import type { Component as ComponentType } from 'vue'
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
  SettingsOutline,
  InformationCircleOutline,
  ChevronDownOutline,
  LayersOutline,
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

// 节点代理模式选择
const currentNodeProxyMode = ref('rule')
const targetNodeProxyMode = ref('')
const showNodeModeChangeModal = ref(false)
const isChangingNodeMode = ref(false)

// 动态渲染图标的辅助函数
function renderIcon(icon: ComponentType) {
  return () => h('div', { class: 'dropdown-option-icon' }, h(icon))
}

// 定义代理模式数据
const proxyModes = [
  {
    value: 'system',
    nameKey: 'home.proxyMode.system',
    tipKey: 'home.proxyMode.systemTip',
    icon: GlobeOutline,
  },
  {
    value: 'manual',
    nameKey: 'home.proxyMode.manual',
    tipKey: 'home.proxyMode.manualTip',
    icon: SettingsOutline,
  },
  {
    value: 'tun',
    nameKey: 'home.proxyMode.tun',
    tipKey: 'home.proxyMode.tunTip',
    icon: FlashOutline,
  },
]

// 定义节点代理模式选项 (更改为数组形式，与proxyModes一致)
const nodeProxyModes = [
  {
    label: t('proxy.mode.global'),
    value: 'global',
    icon: GlobeOutline,
  },
  {
    label: t('proxy.mode.rule'),
    value: 'rule',
    icon: LayersOutline,
  },
]

// 监听appStore中代理模式变化，更新当前选中状态
watch(
  () => appState.proxyMode,
  (newMode) => {
    if (newMode !== currentProxyMode.value) {
      currentProxyMode.value = newMode
    }
  }
)

// 为节点代理模式添加监听
watch(currentNodeProxyMode, (newMode, oldMode) => {
  if (newMode !== oldMode && oldMode) {
    handleNodeProxyModeChange(newMode)
  }
})

// 获取当前节点代理模式
const getCurrentNodeProxyMode = async () => {
  try {
    // 调用后端API获取当前代理模式
    const mode = await tauriApi.proxy.getCurrentProxyMode()
    currentNodeProxyMode.value = mode
  } catch (error) {
    // 出错时仍使用默认的规则模式
    currentNodeProxyMode.value = 'rule'
  }
}

/**
 * 获取节点代理模式对应的文本
 * @param mode 代理模式
 * @returns 模式文本
 */
const getNodeProxyModeText = (mode: string): string => {
  const modeMap: Record<string, string> = {
    global: t('proxy.mode.global'),
    rule: t('proxy.mode.rule'),
  }
  return modeMap[mode] || t('proxy.mode.unknown')
}

/**
 * 处理节点代理模式变更
 */
const handleNodeProxyModeChange = (key: string) => {
  if (key === currentNodeProxyMode.value) return

  // 保存当前选中项，以便用户取消时恢复
  const prevMode = currentNodeProxyMode.value
  targetNodeProxyMode.value = key
  
  // 打开确认对话框
  showNodeModeChangeModal.value = true
  
  // 如果用户取消操作，恢复之前的选择
  const unwatch = watch(showNodeModeChangeModal, (isVisible) => {
    if (!isVisible && !isChangingNodeMode.value) {
      currentNodeProxyMode.value = prevMode
      unwatch() // 取消监听
    }
  })
}

/**
 * 确认切换节点代理模式
 */
const confirmNodeProxyModeChange = async () => {
  if (!targetNodeProxyMode.value) return

  isChangingNodeMode.value = true
  try {
    await tauriApi.proxy.toggleProxyMode(targetNodeProxyMode.value)
    await kernelStore.restartKernel()
    currentNodeProxyMode.value = targetNodeProxyMode.value
    message.success(
      t('proxy.modeChangeSuccess', { mode: getNodeProxyModeText(targetNodeProxyMode.value) }),
    )
  } catch (error) {
    message.error(`${t('proxy.modeChangeError')}: ${error}`)
  } finally {
    isChangingNodeMode.value = false
    showNodeModeChangeModal.value = false
  }
}

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
    // 确保当前模式已设置到appStore
    appState.setProxyMode(currentProxyMode.value)
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
  if (value === currentProxyMode.value) return

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
    // 切换模式
    let needClose = false
    let modeChanged = false

    // 统一使用 proxyService.switchMode 方法切换所有模式
    if (value === 'system' || value === 'manual' || value === 'tun') {
      needClose = await proxyService.switchMode(value, showMessage)
      currentProxyMode.value = value
      modeChanged = true

      // 根据不同模式显示不同的提示信息
      if (value === 'system') {
        showMessage('success', t('notification.systemProxyEnabled'))
      } else if (value === 'manual') {
        showMessage('info', t('notification.manualProxyEnabled'))
      }
    }

    // 如果内核正在运行且模式已改变，一定要重启内核
    if (appState.isRunning && modeChanged) {
      showMessage('info', t('notification.restartingKernel'))
      await kernelStore.restartKernel()
      showMessage('success', t('notification.kernelRestarted'))
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
        connectionStore.setupMemoryListener(),
      ]).catch((e) => {
        // 尝试重试一次
        return new Promise((resolve) => {
          setTimeout(async () => {
            try {
              await trafficStore.setupTrafficListener()
              await connectionStore.setupConnectionsListener()
              await connectionStore.setupMemoryListener()
              resolve(true)
            } catch (retryError) {
              console.error('HomeView: 重试设置监听器失败', retryError)
              resolve(false)
            }
          }, 1000)
        })
      })

      isTrafficLoading.value = false
      isConnectionLoading.value = false
    }
  } catch (error) {
    console.error('HomeView: 设置监听器失败:', error)
    isTrafficLoading.value = false
    isConnectionLoading.value = false
  }
}

onMounted(async () => {
  // 更新当前代理模式
  currentProxyMode.value = appState.proxyMode

  // 获取节点代理模式
  await getCurrentNodeProxyMode()

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
  watch(
    () => appState.isRunning,
    (isRunning) => {
      if (isRunning && isRouteActive.value) {
        setupListeners()
      } else if (!isRunning) {
        // 内核停止时清理监听器
        trafficStore.cleanupListeners()
        connectionStore.cleanupListeners()
      }
    },
  )
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
  padding: 8px 6px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* 状态卡片样式 */
.status-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.04);
}

.status-card :deep(.n-card__content) {
  padding: 8px 16px;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.status-right {
  display: flex;
  gap: 12px;
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

.status-tags {
  display: flex;
  gap: 8px;
}

.status-tag {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 12px;
  border-radius: 6px;
}

.control-button {
  border-radius: 8px;
  font-weight: 500;
}

/* 代理模式卡片 */
.proxy-modes-container {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 10px;
}

.mode-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.04);
  height: 100%;
}

.mode-card :deep(.n-card__content) {
  padding: 12px 16px;
}

.mode-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  padding-bottom: 6px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.1);
}

.mode-card-icon {
  color: var(--primary-color);
}

.mode-card-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.mode-card-content {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mode-buttons {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 移除单选框特定样式，恢复原始样式 */
.mode-button {
  font-weight: 500;
  flex: 1;
  min-width: 100px;
  padding: 4px 10px;
}

.mode-description {
  font-size: 12px;
  color: var(--n-text-color-3);
  line-height: 1.4;
  padding: 2px 0;
  max-height: 40px;
  overflow: hidden;
}

/* 确认对话框 */
.modal-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

.modal-icon {
  color: var(--primary-color);
}

.modal-content {
  margin: 16px 0;
  line-height: 1.6;
}

.modal-footer {
  margin-top: 8px;
}

/* 流量统计卡片 */
.stats-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.04);
}

.stats-card :deep(.n-card__content) {
  padding: 10px 16px;
}

.stats-card :deep(.n-card__header) {
  padding: 8px 16px;
}

.stats-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stats-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.stats-icon {
  color: var(--primary-color);
}

.connections-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: var(--n-text-color-2);
}

.traffic-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.traffic-stats {
  width: 100%;
}

.traffic-row {
  display: flex;
  flex-wrap: nowrap;
  justify-content: space-between;
  gap: 8px;
  overflow-x: auto;
  padding-bottom: 2px; /* 为滚动条预留空间 */
}

/* 隐藏滚动条但保留功能 */
.traffic-row::-webkit-scrollbar {
  height: 4px;
}

.traffic-row::-webkit-scrollbar-thumb {
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 4px;
}

:deep(.dark) .traffic-row::-webkit-scrollbar-thumb {
  background-color: rgba(255, 255, 255, 0.1);
}

.traffic-item {
  flex: 1 0 auto;
  min-width: 120px;
  max-width: 180px;
  background-color: rgba(0, 0, 0, 0.01);
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.1);
  white-space: nowrap;
}

:deep(.dark) .traffic-item {
  background-color: rgba(255, 255, 255, 0.02);
}

.traffic-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--n-text-color-2);
  margin-bottom: 2px;
}

.traffic-icon {
  flex-shrink: 0;
}

.upload-icon {
  color: var(--success-color);
}

.download-icon {
  color: var(--primary-color);
}

.cloud-up-icon {
  color: #2a9d8f;
}

.cloud-down-icon {
  color: #4c6ef5;
}

.memory-icon {
  color: var(--error-color);
}

.traffic-value {
  font-size: 16px;
  font-weight: 600;
  color: var(--n-text-color-1);
  padding-left: 4px;
}

.chart-container {
  width: 100%;
  height: 150px;
  border-radius: 10px;
  overflow: hidden;
  background-color: rgba(0, 0, 0, 0.01);
  margin-top: 4px;
}

:deep(.dark) .chart-container {
  background-color: rgba(255, 255, 255, 0.02);
}

.traffic-chart {
  width: 100%;
  height: 100%;
}

@media (max-width: 820px) {
  .traffic-row {
    flex-wrap: nowrap;
    overflow-x: auto;
  }

  .traffic-item {
    min-width: 110px;
  }
}

@media (max-width: 768px) {
  .status-left,
  .status-right {
    width: 100%;
    justify-content: space-between;
  }

  .status-header {
    flex-direction: column;
    gap: 16px;
  }

  .traffic-content {
    flex-direction: column;
  }

  .traffic-row {
    flex-direction: column;
  }

  .traffic-item {
    min-width: 105px;
    padding: 5px 8px;
  }

  .chart-container {
    height: 130px;
  }
}
</style>
