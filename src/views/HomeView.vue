<template>
  <div class="modern-home">
    <!-- 状态总览区域 -->
    <div class="status-overview">
      <!-- 主状态卡片 -->
      <div class="hero-status-card">
        <div class="status-visual">
          <div class="status-ring" :class="statusRingClass">
            <div class="status-inner">
              <n-icon :size="48" :class="statusIconClass">
                <PowerOutline v-if="!appState.isRunning" />
                <CheckmarkCircleOutline v-else-if="appState.wsConnected" />
                <TimeOutline v-else-if="isStarting || appState.isConnecting" />
                <CloseCircleOutline v-else />
              </n-icon>
            </div>
          </div>
        </div>

        <div class="status-content">
          <h1 class="status-title">{{ getStatusTitle() }}</h1>
          <p class="status-description">{{ getStatusSubtitle() }}</p>

          <!-- 控制按钮 -->
          <div class="control-section">
            <n-button
              v-if="!appState.isRunning"
              type="primary"
              size="large"
              round
              :loading="isStarting"
              @click="runKernel"
              class="primary-action-btn"
            >
              <template #icon>
                <n-icon><PowerOutline /></n-icon>
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
              class="primary-action-btn"
            >
              <template #icon>
                <n-icon><PowerOutline /></n-icon>
              </template>
              {{ t('home.stop') }}
            </n-button>

            <n-button
              v-if="appState.isRunning && !isAdmin && (currentProxyMode === 'tun' || isSwitching)"
              type="warning"
              size="medium"
              round
              :loading="isRestarting"
              @click="restartAsAdmin"
              class="secondary-action-btn"
            >
              <template #icon>
                <n-icon><ShieldOutline /></n-icon>
              </template>
              {{ t('home.restartAsAdmin') }}
            </n-button>
          </div>
        </div>
      </div>

      <!-- 快速统计卡片 -->
      <div class="quick-stats">
        <div class="stats-grid">
          <div class="stat-card" v-for="stat in statsData" :key="stat.key">
            <div class="stat-icon" :class="`stat-${stat.type}`">
              <n-icon :size="20" :component="stat.icon" />
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ stat.value }}</div>
              <div class="stat-label">{{ stat.label }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 配置和图表区域 -->
    <div class="content-grid">
      <!-- 代理配置卡片 -->
      <n-card class="config-card" :bordered="false">
        <template #header>
          <div class="card-header">
            <div class="header-info">
              <n-icon size="20" class="header-icon">
                <LayersOutline />
              </n-icon>
              <span class="header-title">{{ t('home.proxyHeader.flowMode') }}</span>
            </div>
          </div>
        </template>

        <div class="config-content">
          <div class="mode-tabs">
            <div
              v-for="mode in proxyModes"
              :key="mode.value"
              class="mode-tab"
              :class="{
                active: currentProxyMode === mode.value,
                disabled: isSwitching || isStarting || isStopping,
              }"
              @click="!isSwitching && !isStarting && !isStopping && onModeChange(mode.value)"
            >
              <div class="tab-icon">
                <n-icon :size="18" :component="mode.icon" />
              </div>
              <div class="tab-content">
                <div class="tab-title">{{ t(mode.nameKey) }}</div>
                <div class="tab-desc">{{ t(mode.tipKey) }}</div>
              </div>
            </div>
          </div>
        </div>
      </n-card>

      <!-- 节点配置卡片 -->
      <n-card class="config-card" :bordered="false">
        <template #header>
          <div class="card-header">
            <div class="header-info">
              <n-icon size="20" class="header-icon">
                <GitNetworkOutline />
              </n-icon>
              <span class="header-title">{{ t('home.proxyHeader.nodeMode') }}</span>
            </div>
          </div>
        </template>

        <div class="config-content">
          <div class="mode-tabs">
            <div
              v-for="mode in nodeProxyModes"
              :key="mode.value"
              class="mode-tab"
              :class="{
                active: currentNodeProxyMode === mode.value,
                disabled: !appState.isRunning || isSwitching || isStarting || isStopping,
              }"
              @click="
                appState.isRunning &&
                !isSwitching &&
                !isStarting &&
                !isStopping &&
                handleNodeProxyModeChange(mode.value)
              "
            >
              <div class="tab-icon">
                <n-icon :size="18" :component="mode.icon" />
              </div>
              <div class="tab-content">
                <div class="tab-title">{{ mode.label }}</div>
                <div class="tab-desc">{{ t(`proxy.mode.${mode.value}Description`) }}</div>
              </div>
            </div>
          </div>
        </div>
      </n-card>

      <!-- 流量图表卡片 -->
      <n-card class="chart-card" :bordered="false">
        <template #header>
          <div class="card-header">
            <div class="header-info">
              <n-icon size="20" class="header-icon">
                <AnalyticsOutline />
              </n-icon>
              <span class="header-title">{{ t('home.traffic.title') }}</span>
            </div>
            <div class="header-stats">
              <div class="header-stat">
                <n-icon size="14" class="stat-icon upload">
                  <CloudUploadOutline />
                </n-icon>
                <span>{{ formattedTotalUpload }}</span>
              </div>
              <div class="header-stat">
                <n-icon size="14" class="stat-icon download">
                  <CloudDownloadOutline />
                </n-icon>
                <span>{{ formattedTotalDownload }}</span>
              </div>
            </div>
          </div>
        </template>

        <div class="chart-content">
          <TrafficChart
            :upload-speed="trafficStore.traffic.up"
            :download-speed="trafficStore.traffic.down"
          />
        </div>
      </n-card>
    </div>

    <!-- 节点模式切换确认对话框 -->
    <n-modal
      v-model:show="showNodeModeChangeModal"
      preset="dialog"
      :title="t('proxy.confirmSwitch')"
      class="mode-change-modal"
    >
      <div class="modal-content">
        <div class="modal-icon">
          <n-icon size="24" color="#f0a020">
            <InformationCircleOutline />
          </n-icon>
        </div>
        <div class="modal-text">
          {{ t('proxy.switchModeConfirm') }}
        </div>
      </div>
      <template #action>
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
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { useMessage, useDialog } from 'naive-ui'
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { formatBandwidth } from '@/utils'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'
import {
  PowerOutline,
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
  ShieldOutline,
  CloseCircleOutline,
  SettingsOutline,
  InformationCircleOutline,
  LayersOutline,
  CheckmarkCircleOutline,
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores/app/AppStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
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
const themeStore = useThemeStore()
const kernelStore = useKernelStore()
const trafficStore = useTrafficStore()
const connectionStore = useConnectionStore()
const proxyService = ProxyService.getInstance()
const { t } = useI18n()

// 状态变量
const isStarting = ref(false)
const isStopping = ref(false)
const isSwitching = ref(false)
const isRestarting = ref(false)
const isAdmin = ref(false)
const currentProxyMode = ref(appState.proxyMode || 'system')
const currentNodeProxyMode = ref('rule')
const targetNodeProxyMode = ref('')
const showNodeModeChangeModal = ref(false)
const isChangingNodeMode = ref(false)

// 代理模式配置
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

// 计算属性
const route = useRoute()
const isRouteActive = computed(() => route.path === '/')

const statusRingClass = computed(() => {
  if (isStarting.value || appState.isConnecting) return 'status-connecting'
  if (appState.isRunning && appState.wsConnected) return 'status-running'
  if (appState.isRunning && !appState.wsConnected) return 'status-error'
  return 'status-stopped'
})

const statusIconClass = computed(() => {
  if (isStarting.value || appState.isConnecting) return 'icon-connecting'
  if (appState.isRunning && appState.wsConnected) return 'icon-running'
  if (appState.isRunning && !appState.wsConnected) return 'icon-error'
  return 'icon-stopped'
})

// 格式化数据
const formattedUploadSpeed = computed(() => formatBandwidth(Number(trafficStore.traffic.up) || 0))
const formattedDownloadSpeed = computed(() =>
  formatBandwidth(Number(trafficStore.traffic.down) || 0),
)
const formattedTotalUpload = computed(() =>
  formatBandwidth(Number(trafficStore.traffic.totalUp) || 0),
)
const formattedTotalDownload = computed(() =>
  formatBandwidth(Number(trafficStore.traffic.totalDown) || 0),
)
const formattedMemory = computed(() => {
  if (!isRouteActive.value) return '0 B'
  return formatBandwidth(connectionStore.memory?.inuse || 0)
})
const activeConnectionsCount = computed(() => {
  if (!isRouteActive.value) return '0'
  return connectionStore.connections.length.toString()
})

// 统计数据
const statsData = computed(() => [
  {
    key: 'upload',
    type: 'upload',
    icon: ArrowUpOutline,
    value: formattedUploadSpeed.value,
    label: t('home.traffic.uploadSpeed'),
  },
  {
    key: 'download',
    type: 'download',
    icon: ArrowDownOutline,
    value: formattedDownloadSpeed.value,
    label: t('home.traffic.downloadSpeed'),
  },
  {
    key: 'memory',
    type: 'memory',
    icon: HardwareChipOutline,
    value: formattedMemory.value,
    label: t('home.traffic.memory'),
  },
  {
    key: 'connections',
    type: 'connections',
    icon: GitNetworkOutline,
    value: activeConnectionsCount.value,
    label: t('home.traffic.connectionsLabel'),
  },
])

// 状态相关方法
const getStatusTitle = () => {
  if (isStarting.value || appState.isConnecting) return t('home.status.starting')
  if (isStopping.value) return t('home.status.stopping')
  if (appState.isRunning && appState.wsConnected) return t('home.status.running')
  if (appState.isRunning && !appState.wsConnected) return t('home.status.disconnected')
  return t('home.status.stopped')
}

const getStatusSubtitle = () => {
  if (isStarting.value || appState.isConnecting) return t('home.status.startingDesc')
  if (isStopping.value) return t('home.status.stoppingDesc')
  if (appState.isRunning && appState.wsConnected) return t('home.status.runningDesc')
  if (appState.isRunning && !appState.wsConnected) return t('home.status.disconnectedDesc')
  return t('home.status.stoppedDesc')
}

// 核心操作方法 - 补充完整的功能逻辑
const runKernel = async () => {
  try {
    isStarting.value = true
    // 确保当前模式已设置到appStore
    appState.setProxyMode(currentProxyMode.value)

    // 检查TUN模式下是否需要管理员权限
    if (currentProxyMode.value === 'tun') {
      // 每次启动TUN模式时都重新检查管理员权限
      const currentIsAdmin = await tauriApi.system.checkAdmin()

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

// 以管理员权限重启应用
const restartAsAdmin = async () => {
  isRestarting.value = true
  try {
    await tauriApi.system.restartAsAdmin()
  } catch (error) {
    message.error(`${t('notification.restartFailed')}: ${error}`)
    isRestarting.value = false
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
      const isTrafficLoading = ref(false)
      const isConnectionLoading = ref(false)

      isTrafficLoading.value = true
      isConnectionLoading.value = true

      // 使用Promise.all同时设置两个监听器
      await Promise.all([
        trafficStore.setupTrafficListener(),
        connectionStore.setupConnectionsListener(),
        connectionStore.setupMemoryListener(),
      ]).catch((e) => {
        console.error('设置监听器失败，尝试重试', e)
        // 尝试重试一次
        return new Promise((resolve) => {
          setTimeout(async () => {
            try {
              await trafficStore.setupTrafficListener()
              await connectionStore.setupConnectionsListener()
              await connectionStore.setupMemoryListener()
              resolve(true)
            } catch (retryError) {
              console.error('重试设置监听器失败', retryError)
              resolve(false)
            }
          }, 1000)
        })
      })

      isTrafficLoading.value = false
      isConnectionLoading.value = false
    }
  } catch (error) {
    console.error('设置监听器失败:', error)
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

// 监听器
watch(
  () => appState.proxyMode,
  (newMode) => {
    if (newMode !== currentProxyMode.value) {
      currentProxyMode.value = newMode
    }
  },
)

// 生命周期
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
.modern-home {
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 24px;
  min-height: 100%;
  overflow-x: hidden;
}

/* 状态总览区域 */
.status-overview {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 28px;
  margin-bottom: 8px;
  align-items: start;
}

/* 主状态卡片 */
.hero-status-card {
  background: var(--n-card-color);
  border-radius: 20px;
  padding: 32px;
  display: flex;
  align-items: center;
  gap: 32px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  border: 1px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.hero-status-card:hover {
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  transform: translateY(-2px);
}

.status-visual {
  flex-shrink: 0;
}

.status-ring {
  width: 96px;
  height: 96px;
  border-radius: 50%;
  position: relative;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.status-ring.status-running {
  background: conic-gradient(from 0deg, #10b981, #059669, #10b981);
  animation: rotate 3s linear infinite;
}

.status-ring.status-connecting {
  background: conic-gradient(from 0deg, #f59e0b, #d97706, #f59e0b);
  animation: rotate 2s linear infinite;
}

.status-ring.status-error {
  background: conic-gradient(from 0deg, #ef4444, #dc2626, #ef4444);
}

.status-ring.status-stopped {
  background: v-bind('themeStore.isDark ? "rgba(75, 85, 99, 0.3)" : "rgba(156, 163, 175, 0.3)"');
}

.status-inner {
  position: absolute;
  top: 4px;
  left: 4px;
  right: 4px;
  bottom: 4px;
  background: var(--n-card-color);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-running {
  color: #10b981;
}
.icon-connecting {
  color: #f59e0b;
}
.icon-error {
  color: #ef4444;
}
.icon-stopped {
  color: v-bind('themeStore.isDark ? "#9CA3AF" : "#6B7280"');
}

@keyframes rotate {
  to {
    transform: rotate(360deg);
  }
}

.status-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status-title {
  font-size: 28px;
  font-weight: 700;
  margin: 0;
  color: var(--n-text-color);
  line-height: 1.2;
}

.status-description {
  font-size: 16px;
  color: var(--n-text-color-2);
  margin: 0;
  line-height: 1.5;
}

.control-section {
  display: flex;
  gap: 12px;
  align-items: center;
}

.primary-action-btn {
  min-width: 120px;
  height: 44px;
}

.secondary-action-btn {
  height: 36px;
}

/* 快速统计 - 优化布局 */
.quick-stats {
  flex-shrink: 0;
  width: 320px;
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.stat-card {
  background: var(--n-card-color);
  border-radius: 16px;
  padding: 18px 16px;
  border: 1px solid var(--n-border-color);
  display: flex;
  align-items: center;
  gap: 14px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  min-height: 72px;
}

.stat-card:hover {
  border-color: var(--n-primary-color);
  transform: translateY(-1px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
}

.stat-icon {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

/* 改进统计图标配色 - 增强对比度 */
.stat-upload {
  background: v-bind('themeStore.isDark ? "rgba(248, 113, 113, 0.2)" : "rgba(239, 68, 68, 0.1)"');
  color: v-bind('themeStore.isDark ? "#F87171" : "#DC2626"');
}

.stat-download {
  background: v-bind('themeStore.isDark ? "rgba(96, 165, 250, 0.2)" : "rgba(59, 130, 246, 0.1)"');
  color: v-bind('themeStore.isDark ? "#60A5FA" : "#2563EB"');
}

.stat-memory {
  background: v-bind('themeStore.isDark ? "rgba(196, 181, 253, 0.2)" : "rgba(168, 85, 247, 0.1)"');
  color: v-bind('themeStore.isDark ? "#C4B5FD" : "#7C3AED"');
}

.stat-connections {
  background: v-bind('themeStore.isDark ? "rgba(74, 222, 128, 0.2)" : "rgba(34, 197, 94, 0.1)"');
  color: v-bind('themeStore.isDark ? "#4ADE80" : "#16A34A"');
}

.stat-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.stat-value {
  font-size: 15px;
  font-weight: 600;
  color: var(--n-text-color);
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stat-label {
  font-size: 11px;
  color: var(--n-text-color-3);
  margin-top: 3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2;
}

/* 内容网格 */
.content-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

.chart-card {
  grid-column: span 2;
}

/* 卡片样式 */
.config-card,
.chart-card {
  border-radius: 16px !important;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06) !important;
  border: 1px solid var(--n-border-color) !important;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.header-icon {
  color: var(--n-primary-color);
  flex-shrink: 0;
}

.header-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--n-text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-stats {
  display: flex;
  gap: 16px;
  flex-shrink: 0;
}

.header-stat {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--n-text-color-2);
}

.header-stat .stat-icon.upload {
  color: v-bind('themeStore.isDark ? "#F87171" : "#DC2626"');
}

.header-stat .stat-icon.download {
  color: v-bind('themeStore.isDark ? "#60A5FA" : "#2563EB"');
}

/* 配置内容 - 改进配色 */
.config-content {
  padding: 8px 0;
}

.mode-tabs {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mode-tab {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid transparent;
  color: var(--n-text-color);
  min-width: 0;
}

.mode-tab:hover:not(.disabled) {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.04)"');
  border-color: var(--n-border-color);
}

.mode-tab.active {
  background: v-bind('themeStore.isDark ? "rgba(100, 108, 255, 0.2)" : "rgba(100, 108, 255, 0.1)"');
  border-color: var(--n-primary-color);
  color: var(--n-primary-color);
}

.mode-tab.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.tab-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.1)" : "rgba(0, 0, 0, 0.06)"');
  flex-shrink: 0;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.mode-tab.active .tab-icon {
  background: var(--n-primary-color);
  color: white;
  box-shadow: 0 2px 8px rgba(100, 108, 255, 0.3);
}

.mode-tab.active .tab-icon .n-icon {
  color: white !important;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.2));
}

.tab-content {
  flex: 1;
  min-width: 0;
}

.tab-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
  color: inherit;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-desc {
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.6)" : "rgba(0, 0, 0, 0.6)"');
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

/* 图表内容 */
.chart-content {
  height: 280px;
  padding: 16px 0;
}

/* 模态框样式 */
.modal-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}

.modal-icon {
  flex-shrink: 0;
}

.modal-text {
  color: var(--n-text-color-2);
  line-height: 1.6;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .content-grid {
    grid-template-columns: 1fr;
    gap: 20px;
  }

  .chart-card {
    grid-column: span 1;
  }

  .config-card {
    min-width: 0;
  }
}

@media (max-width: 1024px) {
  .quick-stats {
    width: 300px;
  }

  .stats-grid {
    gap: 14px;
  }
}

@media (max-width: 896px) {
  .card-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .header-stats {
    gap: 12px;
    font-size: 12px;
  }

  .mode-tab {
    padding: 14px;
    gap: 10px;
  }

  .tab-icon {
    width: 32px;
    height: 32px;
  }

  .tab-title {
    font-size: 13px;
  }

  .tab-desc {
    font-size: 11px;
    -webkit-line-clamp: 1;
  }
}

@media (max-width: 768px) {
  .status-overview {
    grid-template-columns: 1fr;
    gap: 20px;
  }

  .quick-stats {
    width: 100%;
  }

  .hero-status-card {
    flex-direction: column;
    text-align: center;
    gap: 24px;
    padding: 24px;
  }

  .stats-grid {
    grid-template-columns: repeat(4, 1fr);
    gap: 10px;
  }

  .stat-card {
    flex-direction: column;
    padding: 14px 10px;
    text-align: center;
    min-height: 80px;
    gap: 8px;
  }

  .stat-icon {
    width: 32px;
    height: 32px;
  }

  .stat-value {
    font-size: 14px;
  }

  .stat-label {
    font-size: 10px;
  }

  .config-card,
  .chart-card {
    margin: 0 -8px;
    border-radius: 12px !important;
  }

  .mode-tab {
    padding: 12px;
    gap: 8px;
  }

  .tab-icon {
    width: 28px;
    height: 28px;
  }

  .tab-title {
    font-size: 12px;
  }

  .tab-desc {
    font-size: 10px;
  }

  .header-title {
    font-size: 14px;
  }
}
</style>
