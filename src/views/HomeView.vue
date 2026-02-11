<template>
  <div class="page-container">
    <!-- Header Section -->
    <div class="home-header">
      <div class="header-info">
        <h1 class="page-title">{{ t('nav.home') }}</h1>
        <div class="status-badge" :class="statusClass">
          <div class="status-dot"></div>
          {{ statusTitle }}
        </div>
      </div>
      
      <div class="header-actions">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button
              class="action-button"
              :type="kernelRunning ? 'error' : 'primary'"
              size="large"
              :loading="kernelLoading"
              @click="restartKernel"
            >
              <template #icon>
                <n-icon><PowerOutline /></n-icon>
              </template>
              {{ t('home.restart') }}
            </n-button>
          </template>
          {{ t('home.restart') }}
        </n-tooltip>

        <n-tooltip v-if="isWindowsPlatform && !isAdmin" trigger="hover">
          <template #trigger>
            <n-button
              class="action-button"
              secondary
              type="warning"
              size="large"
              @click="restartAsAdmin"
            >
              <template #icon>
                <n-icon><ShieldCheckmarkOutline /></n-icon>
              </template>
            </n-button>
          </template>
          {{ t('home.restartAsAdmin') }}
        </n-tooltip>
      </div>
    </div>


    <!-- Main Grid -->
    <div class="dashboard-grid">
      <!-- Traffic Stats -->
      <div class="grid-section full-width">
        <div class="stats-row">
          <StatusCard
            :label="t('home.traffic.up')"
            :value="formatSpeed(trafficStore.traffic.up)"
            :description="t('home.traffic.total') + ': ' + formatBytes(trafficStore.traffic.totalUp)"
            type="primary"
          >
            <template #icon><ArrowUpOutline /></template>
          </StatusCard>
          
          <StatusCard
            :label="t('home.traffic.down')"
            :value="formatSpeed(trafficStore.traffic.down)"
            :description="t('home.traffic.total') + ': ' + formatBytes(trafficStore.traffic.totalDown)"
            type="success"
          >
            <template #icon><ArrowDownOutline /></template>
          </StatusCard>

          <StatusCard
            :label="t('nav.connections')"
            :value="connectionStore.connections.length"
            :description="t('home.memory') + ': ' + formatBytes(connectionStore.memory.inuse)"
            type="warning"
          >
            <template #icon><PulseOutline /></template>
          </StatusCard>
        </div>

        
        <!-- Traffic Chart -->
        <div class="chart-section">
          <TrafficChart 
            :upload-speed="trafficStore.traffic.up" 
            :download-speed="trafficStore.traffic.down" 
          />
        </div>
      </div>

      <!-- Proxy Modes -->
      <div class="grid-section">
        <div class="section-header">
          <h3 class="section-title">{{ t('home.proxyHeader.flowMode') }}</h3>
        </div>
        <div class="mode-cards">
          <div 
            class="mode-card" 
            :class="{ active: systemProxyEnabled }"
            @click="toggleSystemProxy(!systemProxyEnabled)"
          >
            <div class="mode-icon">
              <n-icon><GlobeOutline /></n-icon>
            </div>
            <div class="mode-info">
              <div class="mode-name">{{ t('home.proxyMode.system') }}</div>
              <div class="mode-desc">{{ t('home.proxyMode.systemTip') }}</div>
            </div>
            <n-switch :value="systemProxyEnabled" size="small" :disabled="modeSwitchPending" />
          </div>

          <div 
            class="mode-card" 
            :class="{ active: tunProxyEnabled }"
            @click="toggleTunProxy(!tunProxyEnabled)"
          >
            <div class="mode-icon">
              <n-icon><FlashOutline /></n-icon>
            </div>
            <div class="mode-info">
              <div class="mode-name">{{ t('home.proxyMode.tun') }}</div>
              <div class="mode-desc">{{ t('home.proxyMode.tunTip') }}</div>
            </div>
            <n-switch :value="tunProxyEnabled" size="small" :disabled="modeSwitchPending" />
          </div>
        </div>
      </div>

      <!-- Node Modes -->
      <div class="grid-section">
        <div class="section-header">
          <h3 class="section-title">{{ t('home.proxyHeader.nodeMode') }}</h3>
        </div>
        <div class="mode-cards">
          <div 
            v-for="mode in nodeProxyModes"
            :key="mode.value"
            class="mode-card"
            :class="{ active: currentNodeProxyMode === mode.value }"
            @click="handleNodeProxyModeChange(mode.value)"
          >
            <div class="mode-icon">
              <n-icon><component :is="mode.icon" /></n-icon>
            </div>
            <div class="mode-info">
              <div class="mode-name">{{ t(mode.nameKey) }}</div>
              <div class="mode-desc">{{ t(mode.tipKey) }}</div>
            </div>
            <div class="radio-indicator"></div>
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDialog, useMessage, type DialogReactive } from 'naive-ui'
import {
  PowerOutline,
  ShieldCheckmarkOutline,
  ArrowUpOutline,
  ArrowDownOutline,
  PulseOutline,
  GlobeOutline,
  FlashOutline,
  RadioOutline,
  SettingsOutline
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { kernelService } from '@/services/kernel-service'
import StatusCard from '@/components/common/StatusCard.vue'
import TrafficChart from '@/components/layout/TrafficChart.vue'
import { useKernelStatus } from '@/composables/useKernelStatus'
import { useSudoStore } from '@/stores'

defineOptions({
  name: 'HomeView',
})

const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()

// Stores
const appStore = useAppStore()
const kernelStore = useKernelStore()
const trafficStore = useTrafficStore()
const connectionStore = useConnectionStore()
const themeStore = useThemeStore()
const sudoStore = useSudoStore()

// Kernel status (shared with layout)
const { statusClass, statusState, isRunning: kernelRunning, isLoading: kernelLoading } =
  useKernelStatus(kernelStore)
const isAdmin = ref(false)
const platform = ref<'windows' | 'linux' | 'macos' | 'unknown'>('unknown')
const currentNodeProxyMode = ref('rule')
const modeSwitchPending = ref(false)

const isWindowsPlatform = computed(() => platform.value === 'windows')
const isUnixPlatform = computed(() => platform.value === 'linux' || platform.value === 'macos')

const formatBytes = (bytes: number) => {
  if (!bytes) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const value = bytes / Math.pow(1024, index)
  return `${value.toFixed(2)} ${units[index]}`
}

const formatSpeed = (bytes: number) => `${formatBytes(bytes)}/s`

const statusTitle = computed(() => {
  switch (statusState.value) {
    case 'starting':
      return t('status.starting')
    case 'stopping':
      return t('status.stopping')
    case 'running':
      return t('status.running')
    case 'disconnected':
      return t('status.disconnected')
    default:
      return t('status.stopped')
  }
})

const systemProxyEnabled = computed(() => appStore.systemProxyEnabled)
const tunProxyEnabled = computed(() => appStore.tunEnabled)

const nodeProxyModes = [
  {
    value: 'global',
    nameKey: 'home.nodeMode.global',
    tipKey: 'home.nodeMode.globalTip',
    icon: GlobeOutline,
  },
  {
    value: 'rule',
    nameKey: 'home.nodeMode.rule',
    tipKey: 'home.nodeMode.ruleTip',
    icon: RadioOutline,
  },
]

// Methods
const toggleSystemProxy = async (value: boolean) => {
  if (modeSwitchPending.value) return
  
  try {
    modeSwitchPending.value = true
    await appStore.toggleSystemProxy(value)

    const success = await kernelStore.applyProxySettings()
    if (success) {
      message.success(t('notification.proxyModeChanged'))
    } else {
      message.error(kernelStore.lastError || t('notification.proxyModeChangeFailed'))
    }
  } catch (error) {
    message.error(t('notification.proxyModeChangeFailed'))
  } finally {
    modeSwitchPending.value = false
  }
}

const confirmTunSwitch = () => {
  let dialogReactive: DialogReactive | null = null
  let resolved = false

  return new Promise<boolean>((resolve) => {
    const finish = (result: boolean) => {
      if (resolved) return
      resolved = true
      resolve(result)
    }

    const handlePositiveClick = async () => {
      modeSwitchPending.value = true
      if (dialogReactive) dialogReactive.loading = true // 显示加载条，提示正在以管理员方式重启

      try {
        // 非管理员场景下先准备好配置并请求以管理员重启
        const success = await prepareTunModeWithAdminRestart()
        finish(success)
        return success
      } finally {
        modeSwitchPending.value = false
        if (dialogReactive) dialogReactive.loading = false
      }
    }

    dialogReactive = dialog.warning({
      title: t('home.tunConfirm.title'),
      content: t('home.tunConfirm.description'),
      positiveText: t('home.tunConfirm.confirm'),
      negativeText: t('common.cancel'),
      maskClosable: false,
      onPositiveClick: handlePositiveClick,
      onNegativeClick: () => finish(false),
      onClose: () => finish(false),
    })
  })
}

const parseSudoCode = (raw: unknown) => {
  const msg = raw instanceof Error ? raw.message : String(raw || '')
  if (msg.includes('SUDO_PASSWORD_REQUIRED')) return 'required'
  if (msg.includes('SUDO_PASSWORD_INVALID')) return 'invalid'
  return null
}

const getErrorMessage = (error: unknown) => {
  if (error instanceof Error) {
    return error.message
  }
  return String(error || '')
}

const enableTunWithKernelRestart = async (options?: { allowSudoRetry?: boolean }) => {
  try {
    modeSwitchPending.value = true
    await appStore.toggleTun(true)

    const applied = await kernelStore.applyProxySettings()
    if (!applied) {
      await appStore.toggleTun(false)
      message.error(t('notification.proxyModeChangeFailed'))
      return false
    }

    const success = await kernelStore.restartKernel()
    if (success) {
      message.success(t('notification.proxyModeChanged'))
      return true
    }

    await appStore.toggleTun(false)

    // Linux/macOS：sudo 密码缺失/失效时，提示用户重新设置，并允许一次自动重试
    if (isUnixPlatform.value) {
      const code = parseSudoCode(kernelStore.lastError)
      if (code === 'required' || code === 'invalid') {
        message.error(code === 'invalid' ? t('home.sudoPassword.invalid') : t('home.sudoPassword.required'))

        const allowRetry = options?.allowSudoRetry ?? true
        const ok = await sudoStore.requestPassword()
        if (ok && allowRetry) {
          return enableTunWithKernelRestart({ allowSudoRetry: false })
        }
        return false
      }
    }

    message.error(t('home.restartFailed'))
    return false
  } catch (error) {
    await appStore.toggleTun(false)
    message.error(t('notification.proxyModeChangeFailed'))
    return false
  } finally {
    modeSwitchPending.value = false
  }
}

const toggleTunProxy = async (value: boolean) => {
  if (modeSwitchPending.value) return
  
  if (value) {
    if (isWindowsPlatform.value) {
      // Windows：启用TUN模式前先刷新管理员状态，有权限时不弹窗直接处理
      await checkAdmin()

      if (isAdmin.value) {
        await enableTunWithKernelRestart()
      } else {
        await confirmTunSwitch()
      }
    } else if (isUnixPlatform.value) {
      // Linux/macOS：首次启用 TUN 弹窗收集系统密码，后续自动 sudo 提权启动内核
      const { sudoService } = await import('@/services/sudo-service')
      const status = await sudoService.getStatus()
      if (!status.supported) {
        message.error(t('home.sudoPassword.unsupported'))
        return
      }
      if (!status.has_saved) {
        const ok = await sudoStore.requestPassword()
        if (!ok) return
      }
      await enableTunWithKernelRestart()
    } else {
      message.error(t('home.sudoPassword.unsupported'))
    }
  } else {
    // 禁用TUN模式 - 需要重启内核
    try {
      modeSwitchPending.value = true
      await appStore.toggleTun(false)

      const applied = await kernelStore.applyProxySettings()
      if (!applied) {
        await appStore.toggleTun(true)
        message.error(t('notification.proxyModeChangeFailed'))
        return
      }
      
      // TUN模式切换需要重启内核
      const success = await kernelStore.restartKernel()
      if (success) {
        message.success(t('notification.proxyModeChanged'))
      } else {
        // 如果重启失败，恢复状态
        await appStore.toggleTun(true)
        message.error(t('home.restartFailed'))
      }
    } catch (error) {
      await appStore.toggleTun(true)
      message.error(t('notification.proxyModeChangeFailed'))
    } finally {
      modeSwitchPending.value = false
    }
  }
}

const restartKernel = async () => {
  if (kernelLoading.value) return

  try {
    const result = await kernelStore.restartKernel()
    if (result) {
      message.success(t('home.restartSuccess'))
    } else {
      message.error(kernelStore.lastError || t('home.restartFailed'))
    }
  } catch (error) {
    message.error(t('home.restartFailed'))
  }
}

const restartAsAdmin = async () => {
  try {
    await requestRestartAsAdmin()
  } catch (error) {
    const details = getErrorMessage(error)
    message.error(details ? `${t('home.restartFailed')}：${details}` : t('home.restartFailed'))
  }
}

// 已移除switchProxyModeAndRefreshKernel - 不再需要，因为System Proxy和TUN是独立的

const requestRestartAsAdmin = async () => {
  const { systemService } = await import('@/services/system-service')
  await systemService.restartAsAdmin()
}

const prepareTunModeWithAdminRestart = async () => {
  try {
    // 保存TUN启用状态
    await appStore.toggleTun(true)
    const applied = await kernelStore.applyProxySettings()
    if (!applied) {
      await appStore.toggleTun(false)
      message.error(t('notification.proxyModeChangeFailed'))
      return false
    }
    await appStore.saveToBackend()

    // 停止内核
    if (appStore.isRunning) {
      await kernelStore.stopKernel()
    }

    // 重启为管理员
    await requestRestartAsAdmin()
    return true
  } catch (error) {
    await appStore.toggleTun(false)
    const details = getErrorMessage(error)
    message.error(details ? `${t('home.restartFailed')}：${details}` : t('home.restartFailed'))
    return false
  }
}

const handleNodeProxyModeChange = async (mode: string) => {
  if (currentNodeProxyMode.value === mode) return

  try {
    await kernelService.switchNodeProxyMode(mode as 'global' | 'rule')
    currentNodeProxyMode.value = mode
    message.success(t('home.nodeModeChangeSuccess'))
  } catch (error) {
    message.error(t('home.nodeModeChangeFailed'))
  }
}

const checkAdmin = async () => {
  try {
    const { systemService } = await import('@/services/system-service')
    isAdmin.value = await systemService.checkAdmin()
  } catch (error) {
    isAdmin.value = false
  }
}

onMounted(async () => {
  try {
    const { systemService } = await import('@/services/system-service')
    const raw = await systemService.getPlatformInfo()
    platform.value = raw === 'windows' || raw === 'linux' || raw === 'macos' ? raw : 'unknown'
  } catch (error) {
    platform.value = 'unknown'
  }
  checkAdmin()
  await kernelStore.initializeStore()
})
</script>

<style scoped>
.page-container {
  padding: var(--layout-page-padding-y, 16px) var(--layout-page-padding-x, 24px);
  max-width: var(--layout-page-max-width, 1200px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--layout-page-gap, 20px);
}

/* Header */
.home-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.page-title {
  font-size: 32px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: -0.02em;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 600;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.status-badge.running {
  background: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.status-badge.pending {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.status-badge.disconnected {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.status-badge.stopped {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.status-badge.running .status-dot {
  box-shadow: 0 0 8px currentColor;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.action-button {
  border-radius: 12px;
  font-weight: 600;
}

/* Grid */
.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--layout-grid-gap, 24px);
}

.grid-section.full-width {
  grid-column: span 2;
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--layout-grid-gap, 24px);
}

/* Sections */
.section-header {
  margin-bottom: 16px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-tertiary);
  letter-spacing: 0.05em;
  margin: 0;
}

.mode-cards {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.mode-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border-radius: 16px;
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  cursor: pointer;
  transition: all 0.2s ease;
}

.mode-card:hover {
  border-color: var(--border-hover);
  transform: translateX(4px);
}

.mode-card.active {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.05), rgba(99, 102, 241, 0.02));
  border-color: var(--primary-color);
}

.mode-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: 20px;
}

.mode-card.active .mode-icon {
  background: var(--primary-color);
  color: white;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}

.mode-info {
  flex: 1;
}

.mode-name {
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.mode-desc {
  font-size: 12px;
  color: var(--text-secondary);
}

.radio-indicator {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid var(--border-color);
  position: relative;
}

.mode-card.active .radio-indicator {
  border-color: var(--primary-color);
}

.mode-card.active .radio-indicator::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--primary-color);
}

@media (max-width: 768px) {
  .dashboard-grid {
    grid-template-columns: 1fr;
  }
  
  .grid-section.full-width {
    grid-column: span 1;
  }
  
  .stats-row {
    grid-template-columns: 1fr;
  }
}

.chart-section {
  margin-top: 24px;
  height: 200px;
  background: var(--glass-bg);
  border-radius: 16px;
  border: 1px solid var(--glass-border);
  overflow: hidden;
  padding: 16px;
}

</style>
