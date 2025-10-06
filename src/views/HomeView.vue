<template>
  <div class="ultra-home">
    <!-- 超紧凑状态区域 -->
    <div class="status-compact">
      <!-- 主状态显示 -->
      <div class="main-status">
        <div class="status-visual-compact">
          <div class="status-orb" :class="statusOrbClass">
            <div class="orb-core"></div>
            <div class="orb-pulse" v-if="appState.isRunning"></div>
            <div class="orb-icon">
              <n-icon :size="24" :class="statusIconClass">
                <PowerOutline v-if="!appState.isRunning" />
                <CheckmarkCircleOutline v-else-if="appState.wsConnected" />
                <TimeOutline v-else-if="isStarting || appState.isConnecting" />
                <CloseCircleOutline v-else />
              </n-icon>
            </div>
          </div>
        </div>

        <div class="status-info-compact">
          <div class="status-title-compact">{{ getStatusTitle() }}</div>
          <div class="status-desc-compact">{{ getStatusSubtitle() }}</div>

          <!-- 紧凑控制按钮 -->
          <div class="control-compact">
            <n-button
              v-if="!appState.isRunning"
              type="primary"
              size="medium"
              :loading="isStarting"
              @click="runKernel"
            >
              <template #icon>
                <n-icon size="16"><PowerOutline /></n-icon>
              </template>
              {{ t('home.start') }}
            </n-button>

            <n-button v-else type="error" size="medium" :loading="isStopping" @click="stopKernel">
              <template #icon>
                <n-icon size="16"><PowerOutline /></n-icon>
              </template>
              {{ t('home.stop') }}
            </n-button>

            <n-button
              v-if="appState.isRunning && !isAdmin && currentProxyMode === 'tun'"
              type="warning"
              size="small"
              :loading="isRestarting"
              @click="restartAsAdmin"
              class="admin-btn"
            >
              <template #icon>
                <n-icon size="14"><ShieldOutline /></n-icon>
              </template>
              {{ t('home.escalatePrivileges') }}
            </n-button>
          </div>
        </div>
      </div>

      <!-- 实时数据面板 -->
      <div class="realtime-panel">
        <div class="metric-chip" v-for="metric in realtimeMetrics" :key="metric.key">
          <div class="metric-icon" :class="`metric-${metric.type}`">
            <n-icon :size="14" :component="metric.icon" />
          </div>
          <div class="metric-data">
            <div class="metric-value">{{ metric.value }}</div>
            <div class="metric-label">{{ metric.label }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 配置面板 -->
    <div class="config-panel">
      <!-- 代理模式选择 -->
      <div class="mode-selector">
        <div class="selector-header">
          <n-icon size="16" class="selector-icon">
            <LayersOutline />
          </n-icon>
          <span class="selector-title">{{ t('home.proxyModeSelector') }}</span>
        </div>
        <div class="mode-options">
          <div
            v-for="mode in proxyModes"
            :key="mode.value"
            class="mode-option"
            :class="{
              'mode-active': currentProxyMode === mode.value,
              'mode-disabled': isSwitching || isStarting || isStopping,
            }"
            @click="!isSwitching && !isStarting && !isStopping && onModeChange(mode.value)"
          >
            <div class="option-icon">
              <n-icon :size="16" :component="mode.icon" />
            </div>
            <div class="option-content">
              <div class="option-name">{{ t(mode.nameKey) }}</div>
              <div class="option-desc">{{ t(mode.tipKey) }}</div>
            </div>
            <div class="option-indicator"></div>
          </div>
        </div>
      </div>

      <!-- 节点模式选择 -->
      <div class="mode-selector">
        <div class="selector-header">
          <n-icon size="16" class="selector-icon">
            <GitNetworkOutline />
          </n-icon>
          <span class="selector-title">{{ t('home.nodeModeSelector') }}</span>
        </div>
        <div class="mode-options">
          <div
            v-for="mode in nodeProxyModes"
            :key="mode.value"
            class="mode-option"
            :class="{
              'mode-active': currentNodeProxyMode === mode.value,
              'mode-disabled': !appState.isRunning || isSwitching || isStarting || isStopping,
            }"
            @click="
              appState.isRunning &&
              !isSwitching &&
              !isStarting &&
              !isStopping &&
              handleNodeProxyModeChange(mode.value)
            "
          >
            <div class="option-icon">
              <n-icon :size="16" :component="mode.icon" />
            </div>
            <div class="option-content">
              <div class="option-name">{{ mode.label }}</div>
              <div class="option-desc">{{ t(`proxy.mode.${mode.value}Description`) }}</div>
            </div>
            <div class="option-indicator"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- 流量监控 -->
    <div class="traffic-monitor">
      <div class="monitor-header">
        <div class="header-info">
          <n-icon size="16" class="monitor-icon">
            <AnalyticsOutline />
          </n-icon>
          <span class="monitor-title">{{ t('home.trafficMonitor') }}</span>
        </div>
        <div class="traffic-summary">
          <div class="summary-item upload">
            <n-icon size="12"><CloudUploadOutline /></n-icon>
            <span>{{ formattedTotalUpload }}</span>
          </div>
          <div class="summary-item download">
            <n-icon size="12"><CloudDownloadOutline /></n-icon>
            <span>{{ formattedTotalDownload }}</span>
          </div>
        </div>
      </div>
      <div class="chart-container">
        <TrafficChart
          :upload-speed="trafficStore.traffic.up"
          :download-speed="trafficStore.traffic.down"
        />
      </div>
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
import { computed, ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
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
const nodeProxyModeChangeSuccess = ref(false)

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
    tipKey: 'home.nodeMode.globalTip',
    icon: GlobeOutline,
  },
  {
    label: t('proxy.mode.rule'),
    value: 'rule',
    tipKey: 'home.nodeMode.ruleTip',
    icon: LayersOutline,
  },
]

// 计算属性
const route = useRoute()
const isRouteActive = computed(() => route.path === '/')

const statusRingClass = computed(() => {
  if (isStarting.value || appState.isConnecting) return 'status-connecting'

  // 改进状态判断逻辑
  if (appState.isRunning) {
    const hasTrafficData =
      trafficStore.traffic.up > 0 ||
      trafficStore.traffic.down > 0 ||
      trafficStore.traffic.totalUp > 0 ||
      trafficStore.traffic.totalDown > 0
    const hasConnectionData =
      connectionStore.connections.length > 0 || connectionStore.memory.inuse > 0

    if (appState.wsConnected || hasTrafficData || hasConnectionData) {
      return 'status-running'
    } else {
      return 'status-connecting' // 内核运行但正在建立连接
    }
  }

  return 'status-stopped'
})

const statusIconClass = computed(() => {
  if (isStarting.value || appState.isConnecting) return 'icon-connecting'

  // 改进状态判断逻辑
  if (appState.isRunning) {
    const hasTrafficData =
      trafficStore.traffic.up > 0 ||
      trafficStore.traffic.down > 0 ||
      trafficStore.traffic.totalUp > 0 ||
      trafficStore.traffic.totalDown > 0
    const hasConnectionData =
      connectionStore.connections.length > 0 || connectionStore.memory.inuse > 0

    if (appState.wsConnected || hasTrafficData || hasConnectionData) {
      return 'icon-running'
    } else {
      return 'icon-connecting' // 内核运行但正在建立连接
    }
  }

  return 'icon-stopped'
})

// 状态球类计算
const statusOrbClass = computed(() => {
  if (isStarting.value || appState.isConnecting) return 'orb-connecting'
  if (appState.isRunning) return 'orb-running'
  return 'orb-stopped'
})

// 实时指标数据
const realtimeMetrics = computed(() => [
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
    label: t('home.traffic.connections'),
  },
])

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

  // 改进状态判断逻辑：如果内核运行中，优先显示运行状态
  if (appState.isRunning) {
    // 如果有流量数据或连接数据，认为连接正常
    const hasTrafficData =
      trafficStore.traffic.up > 0 ||
      trafficStore.traffic.down > 0 ||
      trafficStore.traffic.totalUp > 0 ||
      trafficStore.traffic.totalDown > 0
    const hasConnectionData =
      connectionStore.connections.length > 0 || connectionStore.memory.inuse > 0

    if (appState.wsConnected || hasTrafficData || hasConnectionData) {
      return t('home.status.running')
    } else {
      // 内核运行但暂无数据，可能正在连接中
      return t('home.status.starting')
    }
  }

  return t('home.status.stopped')
}

const getStatusSubtitle = () => {
  if (isStarting.value || appState.isConnecting) return t('home.status.startingDesc')
  if (isStopping.value) return t('home.status.stoppingDesc')

  // 改进状态判断逻辑
  if (appState.isRunning) {
    const hasTrafficData =
      trafficStore.traffic.up > 0 ||
      trafficStore.traffic.down > 0 ||
      trafficStore.traffic.totalUp > 0 ||
      trafficStore.traffic.totalDown > 0
    const hasConnectionData =
      connectionStore.connections.length > 0 || connectionStore.memory.inuse > 0

    if (appState.wsConnected || hasTrafficData || hasConnectionData) {
      return t('home.status.runningDesc')
    } else {
      return t('home.status.startingDesc') // 正在建立连接
    }
  }

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
              // 先调用API修改配置为TUN模式
              message.info(t('notification.applyingTunMode'))
              await tauriApi.proxy.setTunProxy()

              // 设置应用状态
              appState.setProxyMode('tun')
              currentProxyMode.value = 'tun'

              // 设置挂起的TUN模式标记，重启后会应用（配置已经修改好了）
              localStorage.setItem('pending-tun-mode', 'true')

              message.success(t('notification.tunConfigApplied'))
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

    // 启动内核（简化版本，依赖后端检查）
    try {
      await kernelStore.startKernel()
      message.success(t('notification.kernelStarted'))
      return // 后端确认启动成功
    } catch (startError) {
      // 启动失败，但检查内核进程是否实际在运行
      const isKernelRunning = await tauriApi.kernel.isKernelRunning().catch(() => false)

      if (isKernelRunning) {
        // 内核进程存在，设置为运行状态
        message.warning(t('notification.kernelProcessRunning'))
        appState.setRunningState(true)

        // WebSocket 连接现在由后端自动管理
        console.log('内核进程在运行，WebSocket 连接应该会自动建立')

        return
      }

      // 内核确实没有运行，抛出错误
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
              // 先调用API修改配置为TUN模式
              message.info(t('notification.applyingTunMode'))
              await tauriApi.proxy.setTunProxy()

              // 设置应用状态
              await appState.switchProxyMode('tun')
              currentProxyMode.value = 'tun'

              // 设置挂起的TUN模式标记，重启后会应用（配置已经修改好了）
              localStorage.setItem('pending-tun-mode', 'true')

              message.success(t('notification.tunConfigApplied'))
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

      // 使用Promise.all同时设置监听器
      await Promise.all([
        trafficStore.setupEventListeners(),
        connectionStore.setupEventListeners(),
      ]).catch((e) => {
        console.error('设置监听器失败，尝试重试', e)
        // 尝试重试一次
        return new Promise((resolve) => {
          setTimeout(async () => {
            try {
              await trafficStore.setupEventListeners()
              await connectionStore.setupEventListeners()
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
    console.log('从后端获取的代理模式:', mode)
    currentNodeProxyMode.value = mode
    console.log('前端状态已更新为:', currentNodeProxyMode.value)
  } catch (error) {
    console.error('获取代理模式失败:', error)
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
  nodeProxyModeChangeSuccess.value = false

  // 打开确认对话框
  showNodeModeChangeModal.value = true

  // 如果用户取消操作，恢复之前的选择
  const unwatch = watch(showNodeModeChangeModal, (isVisible) => {
    if (!isVisible && !isChangingNodeMode.value && !nodeProxyModeChangeSuccess.value) {
      // 只有在用户取消时才恢复状态，成功操作时不恢复
      currentNodeProxyMode.value = prevMode
      console.log('用户取消了代理模式切换，恢复到:', prevMode)
    }
    if (!isVisible) {
      unwatch() // 无论什么情况下都要取消监听
    }
  })
}

const confirmNodeProxyModeChange = async () => {
  if (!targetNodeProxyMode.value) return

  console.log('开始切换代理模式到:', targetNodeProxyMode.value)
  isChangingNodeMode.value = true
  try {
    // 1. 先切换后端代理模式配置
    console.log('正在调用后端API切换代理模式...')
    await tauriApi.proxy.toggleProxyMode(targetNodeProxyMode.value)

    // 2. 重启内核以应用新配置
    console.log('正在重启内核...')
    await kernelStore.restartKernel()

    // 3. 内核重启后，从后端重新获取当前代理模式状态，确保前后端状态同步
    console.log('正在同步代理模式状态...')
    await getCurrentNodeProxyMode()

    // 4. 标记操作成功，防止watch监听器恢复状态
    nodeProxyModeChangeSuccess.value = true
    console.log('代理模式切换成功，当前模式:', currentNodeProxyMode.value)

    // 5. 使用nextTick确保DOM更新
    await nextTick()

    message.success(
      t('proxy.modeChangeSuccess', { mode: getNodeProxyModeText(currentNodeProxyMode.value) }),
    )
  } catch (error) {
    console.error('代理模式切换失败:', error)
    message.error(`${t('proxy.modeChangeError')}: ${error}`)
    // 出错时也尝试获取当前状态，避免界面状态不一致
    try {
      await getCurrentNodeProxyMode()
    } catch (syncError) {
      console.error('同步代理模式状态失败:', syncError)
    }
  } finally {
    // 确保状态重置的顺序正确
    showNodeModeChangeModal.value = false
    isChangingNodeMode.value = false
    console.log('代理模式切换操作完成')
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

  // 检查是否有挂起的TUN模式切换（重启后需要应用）
  const pendingTunMode = localStorage.getItem('pending-tun-mode')
  if (pendingTunMode === 'true' && currentProxyMode.value === 'tun') {
    localStorage.removeItem('pending-tun-mode')
    console.log('检测到挂起的TUN模式切换，准备应用配置')

    // 如果当前是管理员权限且内核未运行，则直接启动TUN模式
    if (isAdmin.value && !appState.isRunning) {
      setTimeout(async () => {
        try {
          message.info(t('notification.applyingTunMode'))
          await runKernel()
        } catch (error) {
          message.error(`${t('notification.tunModeApplyFailed')}: ${error}`)
        }
      }, 1000) // 延迟1秒确保界面初始化完成
    }
  }

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
.ultra-home {
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 100%;
  font-size: 13px;
}

/* 超紧凑状态区域 */
.status-compact {
  display: flex;
  gap: 20px;
  align-items: stretch;
  margin-bottom: 4px;
}

/* 主状态显示 */
.main-status {
  flex: 1;
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(12px);
  border: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  border-radius: 12px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 20px;
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.05)"');
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.main-status:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 24px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.25)" : "rgba(0, 0, 0, 0.08)"');
}

/* 状态视觉 */
.status-visual-compact {
  flex-shrink: 0;
}

.status-orb {
  width: 64px;
  height: 64px;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.status-orb.orb-running {
  background: linear-gradient(135deg, #10b981, #059669);
  box-shadow:
    0 0 0 3px rgba(16, 185, 129, 0.2),
    0 0 20px rgba(16, 185, 129, 0.3);
}

.status-orb.orb-connecting {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  box-shadow:
    0 0 0 3px rgba(245, 158, 11, 0.2),
    0 0 20px rgba(245, 158, 11, 0.3);
  animation: orb-pulse 2s ease-in-out infinite;
}

.status-orb.orb-stopped {
  background: v-bind('themeStore.isDark ? "#374151" : "#d1d5db"');
  box-shadow: 0 0 0 3px
    v-bind('themeStore.isDark ? "rgba(55, 65, 81, 0.2)" : "rgba(209, 213, 219, 0.2)"');
}

.orb-core {
  position: absolute;
  width: 56px;
  height: 56px;
  background: v-bind('themeStore.isDark ? "#0f0f10" : "#fafafa"');
  border-radius: 50%;
  z-index: 1;
}

.orb-pulse {
  position: absolute;
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.3);
  animation: pulse-wave 2s ease-in-out infinite;
}

.orb-icon {
  position: relative;
  z-index: 2;
}

@keyframes orb-pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.05);
    opacity: 0.8;
  }
}

@keyframes pulse-wave {
  0% {
    transform: scale(0.8);
    opacity: 0.6;
  }
  50% {
    transform: scale(1.2);
    opacity: 0;
  }
  100% {
    transform: scale(0.8);
    opacity: 0.6;
  }
}

/* 状态信息 */
.status-info-compact {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.status-title-compact {
  font-size: 20px;
  font-weight: 700;
  color: var(--n-text-color);
  margin: 0;
  line-height: 1.2;
}

.status-desc-compact {
  font-size: 13px;
  color: var(--n-text-color-2);
  margin: 0;
  line-height: 1.4;
}

/* 紧凑控制按钮 */
.control-compact {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.start-btn {
  background: linear-gradient(135deg, #10b981, #059669);
  border: none;
  border-radius: 8px;
  font-weight: 600;
  min-width: 80px;
  box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
}

.start-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4);
}

.stop-btn {
  background: linear-gradient(135deg, #ef4444, #dc2626);
  border: none;
  border-radius: 8px;
  font-weight: 600;
  min-width: 80px;
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
}

.stop-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
}

.admin-btn {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  border: none;
  border-radius: 6px;
  font-weight: 600;
  box-shadow: 0 2px 6px rgba(245, 158, 11, 0.3);
}

.admin-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(245, 158, 11, 0.4);
}

/* 实时数据面板 */
.realtime-panel {
  flex: 0 0 280px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  align-content: start;
}

.metric-chip {
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(12px);
  border: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  border-radius: 10px;
  padding: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  min-height: 56px;
}

.metric-chip:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.05)"');
}

.metric-icon {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.metric-upload {
  background: v-bind('themeStore.isDark ? "rgba(248, 113, 113, 0.2)" : "rgba(239, 68, 68, 0.1)"');
  color: v-bind('themeStore.isDark ? "#F87171" : "#DC2626"');
}

.metric-download {
  background: v-bind('themeStore.isDark ? "rgba(96, 165, 250, 0.2)" : "rgba(59, 130, 246, 0.1)"');
  color: v-bind('themeStore.isDark ? "#60A5FA" : "#2563EB"');
}

.metric-memory {
  background: v-bind('themeStore.isDark ? "rgba(196, 181, 253, 0.2)" : "rgba(168, 85, 247, 0.1)"');
  color: v-bind('themeStore.isDark ? "#C4B5FD" : "#7C3AED"');
}

.metric-connections {
  background: v-bind('themeStore.isDark ? "rgba(74, 222, 128, 0.2)" : "rgba(34, 197, 94, 0.1)"');
  color: v-bind('themeStore.isDark ? "#4ADE80" : "#16A34A"');
}

.metric-data {
  flex: 1;
  min-width: 0;
}

.metric-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color);
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.metric-label {
  font-size: 10px;
  color: var(--n-text-color-3);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 配置面板 */
.config-panel {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.mode-selector {
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(12px);
  border: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  border-radius: 12px;
  padding: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.mode-selector:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.05)"');
}

.selector-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
}

.selector-icon {
  color: #6366f1;
  flex-shrink: 0;
}

.selector-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color);
}

.mode-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.mode-option {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid transparent;
  position: relative;
  overflow: hidden;
  min-height: 56px; /* 确保最小高度保持一致 */
}

.mode-option:hover:not(.mode-disabled) {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
  transform: translateX(2px);
}

.mode-option.mode-active {
  background: v-bind('themeStore.isDark ? "rgba(99, 102, 241, 0.15)" : "rgba(99, 102, 241, 0.1)"');
  border-color: #6366f1;
  color: #6366f1;
}

.mode-option.mode-disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.option-icon {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.05)"');
  flex-shrink: 0;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  margin-top: 2px; /* 微调以与文本对齐 */
}

.mode-option.mode-active .option-icon {
  background: #6366f1;
  color: white;
  box-shadow: 0 2px 6px rgba(99, 102, 241, 0.3);
}

.option-content {
  flex: 1;
  min-width: 0;
}

.option-name {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 2px;
  color: inherit;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.option-desc {
  font-size: 11px;
  color: var(--n-text-color-3);
  line-height: 1.3;
  white-space: normal;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  word-break: break-word;
  max-height: 28px; /* 2行文字的最大高度 */
}

.option-indicator {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: linear-gradient(180deg, #6366f1, #8b5cf6);
  opacity: 0;
  transition: opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.mode-option.mode-active .option-indicator {
  opacity: 1;
}

/* 流量监控 */
.traffic-monitor {
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(12px);
  border: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  border-radius: 12px;
  padding: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.traffic-monitor:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.05)"');
}

.monitor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
}

.header-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.monitor-icon {
  color: #6366f1;
  flex-shrink: 0;
}

.monitor-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color);
}

.traffic-summary {
  display: flex;
  gap: 12px;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--n-text-color-2);
}

.summary-item.upload {
  color: v-bind('themeStore.isDark ? "#F87171" : "#DC2626"');
}

.summary-item.download {
  color: v-bind('themeStore.isDark ? "#60A5FA" : "#2563EB"');
}

.chart-container {
  height: 200px;
}

/* 图标状态颜色 */
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

/* 模态框样式 */
.modal-content {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
}

.modal-icon {
  flex-shrink: 0;
}

.modal-text {
  color: var(--n-text-color-2);
  line-height: 1.5;
  font-size: 13px;
}

/* 响应式设计 */
@media (max-width: 1024px) {
  .config-panel {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .realtime-panel {
    flex: 0 0 240px;
  }
}

@media (max-width: 768px) {
  .status-compact {
    flex-direction: column;
    gap: 12px;
  }

  .realtime-panel {
    flex: 1;
    grid-template-columns: repeat(4, 1fr);
  }

  .metric-chip {
    flex-direction: column;
    text-align: center;
    gap: 6px;
    padding: 10px 6px;
    min-height: 64px;
  }

  .main-status {
    padding: 16px;
    gap: 16px;
  }

  .status-orb {
    width: 56px;
    height: 56px;
  }

  .orb-core {
    width: 48px;
    height: 48px;
  }

  .status-title-compact {
    font-size: 18px;
  }

  .control-compact {
    justify-content: center;
  }

  .mode-option {
    min-height: 48px;
    padding: 8px;
  }

  .option-desc {
    font-size: 10px;
    max-height: 24px;
    -webkit-line-clamp: 2;
  }

  .option-icon {
    width: 24px;
    height: 24px;
  }
}

@media (max-width: 480px) {
  .ultra-home {
    gap: 12px;
  }

  .mode-selector,
  .traffic-monitor {
    padding: 12px;
  }

  .chart-container {
    height: 160px;
  }
}
</style>
