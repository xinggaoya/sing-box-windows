<template>
  <div class="home-container">
    <!-- 顶部状态卡片 -->
    <StatusCard 
      :is-running="appState.isRunning"
      :ws-connected="appState.wsConnected"
      :is-admin="isAdmin"
      :is-starting="isStarting"
      :is-stopping="isStopping"
      :is-restarting="isRestarting"
      @start="runKernel"
      @stop="stopKernel"
      @restart-as-admin="restartAsAdmin"
    />

    <!-- 代理模式设置区域 -->
    <div class="proxy-modes-container">
      <!-- 流量代理模式卡片 -->
      <ProxyModeCard
        :title="t('home.proxyHeader.flowMode')"
        :icon="LayersOutline"
        :current-mode="currentProxyMode"
        :modes="proxyModes"
        :disabled="isSwitching || isStarting || isStopping"
        @mode-change="onModeChange"
      />

      <!-- 节点代理模式卡片 -->
      <ProxyModeCard
        :title="t('home.proxyHeader.nodeMode')"
        :icon="GitNetworkOutline"
        :current-mode="currentNodeProxyMode"
        :modes="nodeProxyModes"
        :disabled="!appState.isRunning || isSwitching || isStarting || isStopping"
        :description-prefix="'proxy.mode.'"
        @mode-change="handleNodeProxyModeChange"
      />
    </div>

    <!-- 流量数据卡片 -->
    <TrafficStatsCard 
      :active-connections-count="activeConnectionsCount"
      :traffic-up="trafficStore.traffic.up"
      :traffic-down="trafficStore.traffic.down"
      :total-up="trafficStore.traffic.totalUp"
      :total-down="trafficStore.traffic.totalDown"
      :memory="connectionStore.memory?.inuse || 0"
      :is-route-active="isRouteActive"
    />

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
          <span>{{ t('proxy.switchTo') }}{{ targetNodeProxyMode ? getNodeProxyModeText(targetNodeProxyMode) : '' }}</span>
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

const runKernel = async () => {
  try {
    isStarting.value = true
    // 确保当前模式已设置到appStore
    appState.setProxyMode(currentProxyMode.value)
    
    // 检查TUN模式下是否需要管理员权限
    if (currentProxyMode.value === 'tun' && !isAdmin.value) {
      dialog.warning({
        title: t('notification.adminRequired'),
        content: t('notification.tunModeAdminRequired'),
        positiveText: t('common.restart'),
        negativeText: t('common.cancel'),
        onPositiveClick: async () => {
          await restartAsAdmin()
        }
      })
      isStarting.value = false
      return
    }
    
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
    
    // 检查如果切换到TUN模式且不是管理员权限，则先提示
    if (value === 'tun' && !isAdmin.value) {
      dialog.warning({
        title: t('notification.adminRequired'),
        content: t('notification.tunModeAdminRequired'),
        positiveText: t('common.restart'),
        negativeText: t('common.cancel'),
        onPositiveClick: async () => {
          await restartAsAdmin()
        },
        onNegativeClick: () => {
          // 取消操作，恢复之前的选择
          currentProxyMode.value = appState.proxyMode
          isSwitching.value = false
        }
      })
      return
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
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: fade-in 0.3s ease;
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.proxy-modes-container {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}

/* 确认对话框 */
.node-mode-modal {
  width: 400px;
  max-width: 95vw;
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
}

.modal-footer {
  margin-top: 8px;
}

@media (max-width: 768px) {
  .home-container {
    padding: 12px 8px;
    gap: 12px;
  }
  
  .proxy-modes-container {
    grid-template-columns: 1fr;
    gap: 12px;
  }
}
</style>
