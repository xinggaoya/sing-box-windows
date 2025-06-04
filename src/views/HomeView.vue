<template>
  <div class="home-container">
    <!-- 主状态面板 -->
    <div class="main-status-panel">
      <!-- 核心状态卡片 -->
      <n-card class="status-card" :bordered="false">
        <div class="status-content">
          <!-- 状态指示器 -->
          <div class="status-indicator">
            <div
              :class="[
                'status-pulse',
                {
                  'status-running': appState.isRunning && appState.wsConnected,
                  'status-connecting': appState.isConnecting || isStarting,
                  'status-stopped': !appState.isRunning,
                  'status-error': appState.isRunning && !appState.wsConnected,
                },
              ]"
            >
              <n-icon size="24">
                <power-outline v-if="!appState.isRunning" />
                <checkmark-circle-outline v-else-if="appState.wsConnected" />
                <time-outline v-else-if="appState.isConnecting || isStarting" />
                <close-circle-outline v-else />
              </n-icon>
            </div>
          </div>

          <!-- 状态信息 -->
          <div class="status-info">
            <h2 class="status-title">
              {{ getStatusTitle() }}
            </h2>
            <p class="status-subtitle">
              {{ getStatusSubtitle() }}
            </p>
          </div>

          <!-- 控制按钮组 -->
          <div class="control-buttons">
            <n-button
              v-if="!appState.isRunning"
              type="primary"
              size="large"
              round
              :loading="isStarting"
              @click="runKernel"
              class="control-btn start-btn"
            >
              <template #icon>
                <n-icon><power-outline /></n-icon>
              </template>
              {{ t('home.start') }}
            </n-button>

            <n-button
              v-else
              type="error"
              size="large"
              round
              :loading="isStopping"
              @click="stopKernel"
              class="control-btn stop-btn"
            >
              <template #icon>
                <n-icon><power-outline /></n-icon>
              </template>
              {{ t('home.stop') }}
            </n-button>

            <n-button
              v-if="appState.isRunning && !isAdmin && (currentProxyMode === 'tun' || isSwitching)"
              type="warning"
              size="large"
              round
              :loading="isRestarting"
              @click="restartAsAdmin"
              class="control-btn admin-btn"
            >
              <template #icon>
                <n-icon><shield-outline /></n-icon>
              </template>
              {{ t('home.restartAsAdmin') }}
            </n-button>
          </div>
        </div>
      </n-card>

      <!-- 快速统计网格 -->
      <div class="quick-stats-grid">
        <div class="stat-item">
          <div class="stat-icon upload">
            <n-icon size="20">
              <arrow-up-outline />
            </n-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ formattedUploadSpeed }}</div>
            <div class="stat-label">{{ t('home.traffic.uploadSpeed') }}</div>
          </div>
        </div>

        <div class="stat-item">
          <div class="stat-icon download">
            <n-icon size="20">
              <arrow-down-outline />
            </n-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ formattedDownloadSpeed }}</div>
            <div class="stat-label">{{ t('home.traffic.downloadSpeed') }}</div>
          </div>
        </div>

        <div class="stat-item">
          <div class="stat-icon total-upload">
            <n-icon size="20">
              <cloud-upload-outline />
            </n-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ formattedTotalUpload }}</div>
            <div class="stat-label">{{ t('home.traffic.uploadTotal') }}</div>
          </div>
        </div>

        <div class="stat-item">
          <div class="stat-icon total-download">
            <n-icon size="20">
              <cloud-download-outline />
            </n-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ formattedTotalDownload }}</div>
            <div class="stat-label">{{ t('home.traffic.downloadTotal') }}</div>
          </div>
        </div>

        <div class="stat-item">
          <div class="stat-icon memory">
            <n-icon size="20">
              <hardware-chip-outline />
            </n-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ formattedMemory }}</div>
            <div class="stat-label">{{ t('home.traffic.memory') }}</div>
          </div>
        </div>

        <div class="stat-item">
          <div class="stat-icon connections">
            <n-icon size="20">
              <git-network-outline />
            </n-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ activeConnectionsCount }}</div>
            <div class="stat-label">{{ t('home.traffic.connectionsLabel') }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 配置面板 -->
    <div class="config-panels">
      <!-- 代理模式配置 -->
      <n-card class="config-card proxy-config" :bordered="false">
        <template #header>
          <div class="config-header">
            <n-icon size="18" class="header-icon">
              <layers-outline />
            </n-icon>
            <span>{{ t('home.proxyHeader.flowMode') }}</span>
          </div>
        </template>
        <div class="mode-selector">
          <n-button-group size="medium" class="mode-buttons">
            <n-button
              v-for="mode in proxyModes"
              :key="mode.value"
              :type="currentProxyMode === mode.value ? 'primary' : 'default'"
              :disabled="isSwitching || isStarting || isStopping"
              @click="onModeChange(mode.value)"
              class="mode-btn"
            >
              <template #icon>
                <n-icon>
                  <component :is="mode.icon" />
                </n-icon>
              </template>
              {{ t(mode.nameKey) }}
            </n-button>
          </n-button-group>
          <div class="mode-description">
            {{ t(`home.proxyMode.${currentProxyMode}Tip`) }}
          </div>
        </div>
      </n-card>

      <!-- 节点模式配置 -->
      <n-card class="config-card node-config" :bordered="false">
        <template #header>
          <div class="config-header">
            <n-icon size="18" class="header-icon">
              <git-network-outline />
            </n-icon>
            <span>{{ t('home.proxyHeader.nodeMode') }}</span>
          </div>
        </template>
        <div class="mode-selector">
          <n-button-group size="medium" class="mode-buttons">
            <n-button
              v-for="mode in nodeProxyModes"
              :key="mode.value"
              :type="currentNodeProxyMode === mode.value ? 'primary' : 'default'"
              :disabled="!appState.isRunning || isSwitching || isStarting || isStopping"
              @click="handleNodeProxyModeChange(mode.value)"
              class="mode-btn"
            >
              <template #icon>
                <n-icon>
                  <component :is="mode.icon" />
                </n-icon>
              </template>
              {{ mode.label }}
            </n-button>
          </n-button-group>
          <div class="mode-description">
            {{ t(`proxy.mode.${currentNodeProxyMode}Description`) }}
          </div>
        </div>
      </n-card>
    </div>

    <!-- 流量图表 -->
    <n-card class="chart-card" :bordered="false">
      <template #header>
        <div class="chart-header">
          <div class="header-left">
            <n-icon size="18" class="header-icon">
              <analytics-outline />
            </n-icon>
            <span>{{ t('home.traffic.title') }}</span>
          </div>
          <div class="total-stats">
            <div class="total-item">
              <n-icon size="14" class="total-icon">
                <cloud-upload-outline />
              </n-icon>
              <span>{{ formattedTotalUpload }}</span>
            </div>
            <div class="total-item">
              <n-icon size="14" class="total-icon">
                <cloud-download-outline />
              </n-icon>
              <span>{{ formattedTotalDownload }}</span>
            </div>
          </div>
        </div>
      </template>
      <div class="chart-container">
        <TrafficChart
          :upload-speed="trafficStore.traffic.up"
          :download-speed="trafficStore.traffic.down"
        />
      </div>
    </n-card>

    <!-- 节点模式切换确认对话框 -->
    <n-modal
      v-model:show="showNodeModeChangeModal"
      preset="dialog"
      :title="`${t('proxy.switchTo')}${targetNodeProxyMode ? getNodeProxyModeText(targetNodeProxyMode) : ''}`"
      class="node-mode-modal"
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
  </div>
</template>

<script setup lang="ts">
import { useMessage, useDialog } from 'naive-ui'
import { computed, ref, onMounted, onUnmounted, watch, h } from 'vue'
import { useRoute } from 'vue-router'
import { formatBandwidth } from '@/utils'
import { Window } from '@tauri-apps/api/window'
import type { Component as ComponentType } from 'vue'
import mitt from '@/utils/mitt'
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
  RefreshOutline,
  CheckmarkCircleOutline,
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores/app/AppStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import TrafficChart from '@/components/layout/TrafficChart.vue'
import { ProxyService } from '@/services/proxy-service'
import { useI18n } from 'vue-i18n'
import { tauriApi } from '@/services/tauri-api'

// 导入新拆分的组件
import StatusCard from '@/components/home/StatusCard.vue'
import ProxyModeCard from '@/components/home/ProxyModeCard.vue'
import TrafficStatsCard from '@/components/home/TrafficStatsCard.vue'

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
const isRestarting = ref(false)
const { t } = useI18n()
const isAdmin = ref(false)

// 代理模式选择
const currentProxyMode = ref(appState.proxyMode || 'system')

// 节点代理模式选择
const currentNodeProxyMode = ref('rule')
const targetNodeProxyMode = ref('')
const showNodeModeChangeModal = ref(false)
const isChangingNodeMode = ref(false)

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

// 定义节点代理模式选项
const nodeProxyModes = [
  {
    label: t('proxy.mode.global'),
    value: 'global',
    icon: GlobeOutline,
    nameKey: 'proxy.mode.global',
  },
  {
    label: t('proxy.mode.rule'),
    value: 'rule',
    icon: LayersOutline,
    nameKey: 'proxy.mode.rule',
  },
]

// 格式化流量数据的computed属性
const formattedUploadSpeed = computed(() => {
  if (!isRouteActive.value) return '0 B/s'
  return formatBandwidth(Number(trafficStore.traffic.up) || 0)
})

const formattedDownloadSpeed = computed(() => {
  if (!isRouteActive.value) return '0 B/s'
  return formatBandwidth(Number(trafficStore.traffic.down) || 0)
})

const formattedTotalUpload = computed(() => {
  if (!isRouteActive.value) return '0 B'
  return formatBandwidth(Number(trafficStore.traffic.totalUp) || 0)
})

const formattedTotalDownload = computed(() => {
  if (!isRouteActive.value) return '0 B'
  return formatBandwidth(Number(trafficStore.traffic.totalDown) || 0)
})

const formattedMemory = computed(() => {
  if (!isRouteActive.value) return '0 B'
  return formatBandwidth(connectionStore.memory?.inuse || 0)
})

// 状态相关方法
const getStatusTitle = () => {
  if (isStarting.value || appState.isConnecting) {
    return t('home.status.starting')
  }
  if (isStopping.value) {
    return t('home.status.stopping')
  }
  if (appState.isRunning && appState.wsConnected) {
    return t('home.status.running')
  }
  if (appState.isRunning && !appState.wsConnected) {
    return t('home.status.disconnected')
  }
  return t('home.status.stopped')
}

const getStatusSubtitle = () => {
  if (isStarting.value || appState.isConnecting) {
    return t('home.status.startingDesc')
  }
  if (isStopping.value) {
    return t('home.status.stoppingDesc')
  }
  if (appState.isRunning && appState.wsConnected) {
    return t('home.status.runningDesc')
  }
  if (appState.isRunning && !appState.wsConnected) {
    return t('home.status.disconnectedDesc')
  }
  return t('home.status.stoppedDesc')
}

// 监听路由可见性变化
const route = useRoute()
const isRouteActive = computed(() => route.path === '/')

// 添加加载状态
const isTrafficLoading = ref(false)
const isConnectionLoading = ref(false)

// 保留计算属性的可见性检查，但简化逻辑
const activeConnectionsCount = computed(() => {
  if (!isRouteActive.value) return '0'
  return connectionStore.connections.length.toString()
})

// 监听appStore中代理模式变化，更新当前选中状态
watch(
  () => appState.proxyMode,
  (newMode) => {
    if (newMode !== currentProxyMode.value) {
      currentProxyMode.value = newMode
    }
  },
)

// 为节点代理模式添加监听
watch(currentNodeProxyMode, (newMode, oldMode) => {
  if (newMode !== oldMode && oldMode) {
    handleNodeProxyModeChange(newMode)
  }
})

const runKernel = async () => {
  try {
    isStarting.value = true
    // 确保当前模式已设置到appStore
    appState.setProxyMode(currentProxyMode.value)

    // 检查TUN模式下是否需要管理员权限
    if (currentProxyMode.value === 'tun') {
      // 每次启动TUN模式时都重新检查管理员权限
      const currentIsAdmin = await tauriApi.system.checkAdmin()
      console.log('启动TUN模式 - 当前管理员权限状态:', currentIsAdmin)

      if (!currentIsAdmin) {
        dialog.warning({
          title: t('notification.adminRequired'),
          content: t('notification.tunModeAdminRequired'),
          positiveText: t('common.restart'),
          negativeText: t('common.cancel'),
          onPositiveClick: async () => {
            try {
              // 先设置模式到应用状态，以便重启后保持选择
              appState.setProxyMode('tun')
              currentProxyMode.value = 'tun'
              await restartAsAdmin()
            } catch (error) {
              message.error(`${t('notification.restartFailed')}: ${error}`)
            }
          },
        })
        isStarting.value = false
        return
      }
    }

    // 显示启动中提示
    message.info(t('notification.startingKernel'))

    // 监听启动失败事件
    const onStartFailed = (event: { error: string }) => {
      message.error(event.error)
      mitt.off('kernel-start-failed', onStartFailed)
    }
    mitt.on('kernel-start-failed', onStartFailed)

    // 监听连接状态变化
    const onConnectionChange = (isConnecting: boolean) => {
      if (isConnecting) {
        message.info(t('notification.connectingToKernel'))
      }
    }
    mitt.on('connecting-status-changed', onConnectionChange)

    // 尝试启动内核
    try {
      await kernelStore.startKernel()
      message.success(t('notification.kernelStarted'))
      return // 成功启动则直接返回
    } catch (startError) {
      // 启动失败，检查内核是否已经在运行
      const isKernelRunning = await tauriApi.kernel.isKernelRunning().catch(() => false)

      if (isKernelRunning) {
        // 内核已经在运行，但可能WebSocket连接有问题
        message.info(t('notification.kernelAlreadyRunning'))

        // 设置内核运行状态为true
        appState.setRunningState(true)

        // 尝试一次WebSocket连接
        if (!appState.wsConnected) {
          message.info(t('notification.tryingToConnectWebSocket'))

          // 禁用WebSocket重试，避免循环
          const wsConnected = await kernelStore.setupWebsocketConnection().catch(() => false)

          if (wsConnected) {
            message.success(t('notification.webSocketConnected'))
          } else {
            message.warning(t('notification.webSocketConnectionFailed'))
            // 即使WebSocket连接失败，仍然保持内核运行状态
          }
        }

        return // 内核运行状态已设置，直接返回
      }

      // 如果内核不在运行，继续抛出错误让catch处理
      throw startError
    }
  } catch (error) {
    // 处理已知错误
    let errorMessage =
      typeof error === 'string'
        ? error
        : error instanceof Error
          ? error.message
          : t('notification.unknownError')

    // 如果错误信息太长，截取一部分
    if (errorMessage.length > 150) {
      errorMessage = errorMessage.substring(0, 150) + '...'
    }

    // 显示错误并带有详细说明
    dialog.error({
      title: t('notification.startFailed'),
      content: `${errorMessage}\n\n${t('notification.checkTheFollowing')}:\n1. ${t('notification.checkConfig')}\n2. ${t('notification.checkNetwork')}\n3. ${t('notification.checkPermissions')}`,
      positiveText: t('common.ok'),
    })

    // 确保内核状态设为关闭
    appState.setRunningState(false)
  } finally {
    isStarting.value = false
    // 清理事件监听
    mitt.off('kernel-start-failed')
    mitt.off('connecting-status-changed')
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

    // 检查如果切换到TUN模式且不是管理员权限，则先提示
    if (value === 'tun') {
      // 每次切换TUN模式时都重新检查管理员权限
      const currentIsAdmin = await tauriApi.system.checkAdmin()
      console.log('当前管理员权限状态:', currentIsAdmin)

      if (!currentIsAdmin) {
        dialog.warning({
          title: t('notification.adminRequired'),
          content: t('notification.tunModeAdminRequired'),
          positiveText: t('common.restart'),
          negativeText: t('common.cancel'),
          onPositiveClick: async () => {
            try {
              // 先设置模式到应用状态，以便重启后保持选择
              appState.setProxyMode('tun')
              currentProxyMode.value = 'tun'
              await restartAsAdmin()
            } catch (error) {
              message.error(`${t('notification.restartFailed')}: ${error}`)
            }
          },
          onNegativeClick: () => {
            // 取消操作，恢复之前的选择
            currentProxyMode.value = appState.proxyMode
          },
        })
        return // 直接返回，不继续执行切换操作
      }
    }

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
      } else if (value === 'tun') {
        showMessage('success', t('notification.tunModeEnabled'))
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
    isAdmin.value = false
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

/**
 * 以管理员权限重启应用
 */
const restartAsAdmin = async () => {
  isRestarting.value = true
  try {
    await tauriApi.system.restartAsAdmin()
  } catch (error) {
    message.error(`${t('notification.restartFailed')}: ${error}`)
    isRestarting.value = false
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
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 24px;
  animation: fade-in 0.5s ease-out;
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 主状态面板 */
.main-status-panel {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.status-card {
  border-radius: 20px;
  background: linear-gradient(
    135deg,
    var(--card-color) 0%,
    rgba(var(--primary-color-rgb), 0.02) 100%
  );
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(16px);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.status-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.12);
}

.status-content {
  display: flex;
  align-items: center;
  padding: 32px;
  gap: 32px;
}

.status-indicator {
  position: relative;
  flex-shrink: 0;
}

.status-pulse {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  backdrop-filter: blur(8px);
  color: white;
  font-weight: 600;
}

.status-pulse::before {
  content: '';
  position: absolute;
  inset: -8px;
  border-radius: 50%;
  opacity: 0;
  transition: all 0.4s ease;
}

.status-running {
  background: linear-gradient(135deg, #18a058, #36ad6a);
  box-shadow: 0 8px 24px rgba(24, 160, 88, 0.3);
}

.status-running::before {
  background: linear-gradient(135deg, #18a058, #36ad6a);
  opacity: 0.2;
  animation: pulse 2s infinite;
}

.status-connecting {
  background: linear-gradient(135deg, #f0a020, #faad14);
  box-shadow: 0 8px 24px rgba(240, 160, 32, 0.3);
}

.status-connecting::before {
  background: linear-gradient(135deg, #f0a020, #faad14);
  opacity: 0.2;
  animation: pulse 1.5s infinite;
}

.status-stopped {
  background: linear-gradient(135deg, #666, #888);
  box-shadow: 0 8px 24px rgba(102, 102, 102, 0.2);
}

.status-error {
  background: linear-gradient(135deg, #d03050, #e84749);
  box-shadow: 0 8px 24px rgba(208, 48, 80, 0.3);
}

.status-error::before {
  background: linear-gradient(135deg, #d03050, #e84749);
  opacity: 0.2;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 0.2;
  }
  50% {
    transform: scale(1.1);
    opacity: 0.1;
  }
}

.status-info {
  flex: 1;
  min-width: 0;
}

.status-title {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: var(--text-color);
  letter-spacing: -0.5px;
}

.status-subtitle {
  font-size: 16px;
  color: var(--text-color-2);
  margin: 0;
  opacity: 0.8;
}

.control-buttons {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.control-btn {
  min-width: 140px;
  height: 48px;
  border-radius: 24px;
  font-weight: 600;
  font-size: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  border: none !important;
}

.control-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}

.start-btn {
  background: linear-gradient(135deg, #4080ff, #2266dd) !important;
  color: white !important;
}

.start-btn :deep(.n-button__content) {
  color: white !important;
}

.start-btn :deep(.n-icon) {
  color: white !important;
}

.stop-btn {
  background: linear-gradient(135deg, #f53f3f, #cb2a2a) !important;
  color: white !important;
}

.stop-btn :deep(.n-button__content) {
  color: white !important;
}

.stop-btn :deep(.n-icon) {
  color: white !important;
}

.admin-btn {
  background: linear-gradient(135deg, #ff7d00, #d66600) !important;
  color: white !important;
}

.admin-btn :deep(.n-button__content) {
  color: white !important;
}

.admin-btn :deep(.n-icon) {
  color: white !important;
}

/* 快速统计网格 */
.quick-stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

@media (min-width: 1200px) {
  .quick-stats-grid {
    grid-template-columns: repeat(6, 1fr);
  }
}

@media (min-width: 900px) and (max-width: 1199px) {
  .quick-stats-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

.stat-item {
  background: var(--card-color);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(8px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.04);
  min-width: 0;
}

.stat-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
  border-color: rgba(var(--primary-color-rgb), 0.2);
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  position: relative;
  overflow: hidden;
}

.stat-icon::before {
  content: '';
  position: absolute;
  inset: 0;
  background: inherit;
  opacity: 0.1;
  border-radius: inherit;
}

.upload {
  background: linear-gradient(135deg, #f53f3f, #cb2a2a);
  color: white;
}

.download {
  background: linear-gradient(135deg, #4080ff, #2266dd);
  color: white;
}

.total-upload {
  background: linear-gradient(135deg, #ff7d00, #d66600);
  color: white;
}

.total-download {
  background: linear-gradient(135deg, #00b42a, #009a1a);
  color: white;
}

.memory {
  background: linear-gradient(135deg, #909399, #7b7e83);
  color: white;
}

.connections {
  background: linear-gradient(135deg, #ff7d00, #d66600);
  color: white;
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-color);
  margin-bottom: 4px;
  letter-spacing: -0.3px;
}

.stat-label {
  font-size: 13px;
  color: var(--text-color-2);
  opacity: 0.8;
  font-weight: 500;
}

/* 配置面板 */
.config-panels {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 20px;
}

.config-card {
  border-radius: 16px;
  background: var(--card-color);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.config-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

.config-header {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
  font-size: 16px;
  color: var(--text-color);
}

.header-icon {
  color: var(--primary-color);
  opacity: 0.8;
}

.mode-selector {
  padding: 0;
}

.mode-buttons {
  width: 100%;
  margin-bottom: 16px;
}

.mode-buttons :deep(.n-button-group) {
  width: 100%;
}

.mode-btn {
  flex: 1;
  border-radius: 8px !important;
  font-weight: 500;
  height: 40px;
  transition: all 0.2s ease;
}

.mode-btn:hover:not(:disabled) {
  transform: translateY(-1px);
}

.mode-description {
  font-size: 13px;
  color: var(--text-color-2);
  line-height: 1.5;
  opacity: 0.8;
  padding: 12px 16px;
  background: rgba(var(--primary-color-rgb), 0.04);
  border-radius: 8px;
  margin-top: 8px;
}

/* 图表卡片 */
.chart-card {
  border-radius: 16px;
  background: var(--card-color);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.chart-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

.chart-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
  font-size: 16px;
  color: var(--text-color);
}

.total-stats {
  display: flex;
  gap: 20px;
}

.total-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-color-2);
  font-weight: 500;
}

.total-icon {
  opacity: 0.7;
}

.chart-container {
  height: 200px;
  margin-top: 16px;
}

/* 确认对话框 */
.node-mode-modal {
  max-width: 95vw;
  border-radius: 16px;
}

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
  color: var(--text-color-2);
}

.modal-footer {
  margin-top: 8px;
}

/* 深色模式样式会通过CSS变量自动应用，删除手动适配代码 */

/* 响应式设计 */
@media (max-width: 768px) {
  .home-container {
    padding: 16px 12px;
    gap: 20px;
  }

  .status-content {
    flex-direction: column;
    text-align: center;
    padding: 24px 20px;
    gap: 20px;
  }

  .quick-stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .stat-item {
    padding: 16px;
    flex-direction: column;
    text-align: center;
    gap: 12px;
  }

  .config-panels {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  .control-buttons {
    width: 100%;
    justify-content: center;
  }

  .control-btn {
    min-width: 120px;
    height: 44px;
    font-size: 14px;
  }

  .status-title {
    font-size: 24px;
  }

  .status-subtitle {
    font-size: 14px;
  }

  .chart-container {
    height: 160px;
  }
}

@media (max-width: 640px) {
  .quick-stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }

  .stat-item {
    padding: 12px;
  }

  .stat-icon {
    width: 40px;
    height: 40px;
  }

  .stat-value {
    font-size: 16px;
  }

  .stat-label {
    font-size: 12px;
  }
}

@media (max-width: 480px) {
  .quick-stats-grid {
    grid-template-columns: 1fr;
  }

  .total-stats {
    flex-direction: column;
    gap: 8px;
    align-items: flex-end;
  }
}
</style>
