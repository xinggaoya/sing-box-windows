<template>
  <div class="page-container">
    <div class="home-header">
      <div class="header-info">
        <div class="status-hero" :class="statusClass">
          <div class="status-indicator"></div>
          <div class="status-text">
            <div class="status-label">{{ statusTitle }}</div>
          </div>
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

    <n-alert
      v-if="kernelStore.startupDiagnosis"
      type="error"
      class="diagnosis-alert"
      :title="kernelStore.startupDiagnosis.message"
    >
      <div class="diagnosis-body">
        <div class="diagnosis-meta">
          <n-tag size="small" type="error">{{ kernelStore.startupDiagnosis.stage }}</n-tag>
          <n-tag size="small">{{ kernelStore.startupDiagnosis.kind }}</n-tag>
        </div>
        <div class="diagnosis-detail">{{ kernelStore.startupDiagnosis.detail }}</div>
        <ul
          v-if="kernelStore.startupDiagnosis.suggested_actions?.length"
          class="diagnosis-actions"
        >
          <li v-for="action in kernelStore.startupDiagnosis.suggested_actions" :key="action">
            {{ action }}
          </li>
        </ul>
      </div>
    </n-alert>

    <div class="dashboard-grid">
      <div class="grid-section full-width">
        <div class="stats-row">
          <StatusCard
            :label="t('home.traffic.up')"
            :value="formatSpeed(trafficStore.traffic.up)"
            :description="
              t('home.traffic.total') + ': ' + formatBytes(trafficStore.traffic.totalUp)
            "
            type="primary"
          >
            <template #icon><ArrowUpOutline /></template>
          </StatusCard>

          <StatusCard
            :label="t('home.traffic.down')"
            :value="formatSpeed(trafficStore.traffic.down)"
            :description="
              t('home.traffic.total') + ': ' + formatBytes(trafficStore.traffic.totalDown)
            "
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

          <StatusCard
            :label="t('proxy.title')"
            :value="proxyStore.groupCount"
            :description="`${t('proxy.dashboard.nodeTotal')}: ${proxyStore.nodeCount}`"
            type="default"
          >
            <template #icon><GlobeOutline /></template>
          </StatusCard>
        </div>

        <div class="chart-section">
          <TrafficChart
            :upload-speed="trafficStore.traffic.up"
            :download-speed="trafficStore.traffic.down"
          />
        </div>
      </div>

      <div class="grid-section">
        <div class="card-header">
          <div class="card-header-icon port-icon">
            <n-icon :size="16"><SettingsOutline /></n-icon>
          </div>
          <h3 class="card-title">{{ t('home.proxyPort.title') }}</h3>
          <n-button size="tiny" quaternary @click="showPortModal = true">
            {{ t('common.edit') }}
          </n-button>
        </div>
        <div class="proxy-port-card">
          <div class="port-row">
            <div class="port-protocol">HTTP</div>
            <code class="port-address">{{ proxyAddress }}</code>
          </div>
          <div class="port-row">
            <div class="port-protocol">SOCKS5</div>
            <code class="port-address">{{ proxyAddress }}</code>
          </div>
        </div>
      </div>

      <div class="grid-section">
        <div class="card-header">
          <div class="card-header-icon flow-icon">
            <n-icon :size="16"><SwapVerticalOutline /></n-icon>
          </div>
          <h3 class="card-title">{{ t('home.proxyHeader.flowMode') }}</h3>
        </div>
        <div class="mode-cards">
          <div
            class="mode-card"
            :class="{ active: systemProxyEnabled }"
            @click="toggleSystemProxy(!systemProxyEnabled)"
          >
            <div class="mode-card-icon" :class="{ active: systemProxyEnabled }">
              <n-icon :size="20"><GlobeOutline /></n-icon>
            </div>
            <div class="mode-card-info">
              <div class="mode-card-name">{{ t('home.proxyMode.system') }}</div>
              <div class="mode-card-desc">{{ t('home.proxyMode.systemTip') }}</div>
            </div>
            <n-switch :value="systemProxyEnabled" size="small" :disabled="modeSwitchPending" />
          </div>

          <div
            class="mode-card"
            :class="{ active: tunProxyEnabled }"
            @click="toggleTunProxy(!tunProxyEnabled)"
          >
            <div class="mode-card-icon" :class="{ active: tunProxyEnabled }">
              <n-icon :size="20"><FlashOutline /></n-icon>
            </div>
            <div class="mode-card-info">
              <div class="mode-card-name">{{ t('home.proxyMode.tun') }}</div>
              <div class="mode-card-desc">{{ t('home.proxyMode.tunTip') }}</div>
            </div>
            <n-switch :value="tunProxyEnabled" size="small" :disabled="modeSwitchPending" />
          </div>
        </div>
      </div>

      <div class="grid-section">
        <div class="card-header">
          <div class="card-header-icon node-icon">
            <n-icon :size="16"><RadioOutline /></n-icon>
          </div>
          <h3 class="card-title">{{ t('home.proxyHeader.nodeMode') }}</h3>
        </div>
        <div class="mode-cards">
          <div
            v-for="mode in nodeProxyModes"
            :key="mode.value"
            class="mode-card"
            :class="{ active: currentNodeProxyMode === mode.value }"
            @click="handleNodeProxyModeChange(mode.value)"
          >
            <div class="mode-card-icon" :class="{ active: currentNodeProxyMode === mode.value }">
              <n-icon :size="20"><component :is="mode.icon" /></n-icon>
            </div>
            <div class="mode-card-info">
              <div class="mode-card-name">{{ t(mode.nameKey) }}</div>
              <div class="mode-card-desc">{{ t(mode.tipKey) }}</div>
            </div>
            <div class="radio-dot" :class="{ active: currentNodeProxyMode === mode.value }"></div>
          </div>
        </div>
      </div>
    </div>

    <PortSettingsDialog v-model:show="showPortModal" />
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
  SettingsOutline,
  SwapVerticalOutline,
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { kernelService } from '@/services/kernel-service'
import { proxyService } from '@/services/proxy-service'
import { sudoService } from '@/services/sudo-service'
import { systemService } from '@/services/system-service'
import StatusCard from '@/components/common/StatusCard.vue'
import PortSettingsDialog from '@/components/common/PortSettingsDialog.vue'
import TrafficChart from '@/components/layout/TrafficChart.vue'
import { useKernelStatus } from '@/composables/useKernelStatus'
import { useSudoStore } from '@/stores'
import { formatBytes, formatSpeed } from '@/utils'

defineOptions({
  name: 'HomeView',
})

const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()

const appStore = useAppStore()
const kernelStore = useKernelStore()
const trafficStore = useTrafficStore()
const connectionStore = useConnectionStore()
const proxyStore = useProxyStore()
const themeStore = useThemeStore()
const sudoStore = useSudoStore()

const {
  statusClass,
  statusState,
  isRunning: kernelRunning,
  isLoading: kernelLoading,
} = useKernelStatus(kernelStore)
const isAdmin = ref(false)
const platform = ref<'windows' | 'linux' | 'macos' | 'unknown'>('unknown')
const currentNodeProxyMode = ref('rule')
const modeSwitchPending = ref(false)
const showPortModal = ref(false)

const isWindowsPlatform = computed(() => platform.value === 'windows')
const isUnixPlatform = computed(() => platform.value === 'linux' || platform.value === 'macos')

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
    case 'failed':
      return t('status.failed')
    case 'crashed':
      return t('status.crashed')
    default:
      return t('status.stopped')
  }
})

const systemProxyEnabled = computed(() => appStore.systemProxyEnabled)
const tunProxyEnabled = computed(() => appStore.tunEnabled)
const proxyAddress = computed(() => `127.0.0.1:${appStore.proxyPort}`)

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

const getKernelFailureText = (fallback: string) =>
  kernelStore.startupDiagnosisSummary || kernelStore.lastError || fallback

const syncCurrentNodeProxyMode = async () => {
  try {
    const mode = await proxyService.getCurrentProxyMode()
    if (mode === 'global' || mode === 'rule') {
      currentNodeProxyMode.value = mode
    }
  } catch (error) {
    // 后端未就绪时保持当前 UI 状态，避免启动阶段闪烁成错误模式。
  }
}

const toggleSystemProxy = async (value: boolean) => {
  if (modeSwitchPending.value) return

  try {
    modeSwitchPending.value = true
    await appStore.toggleSystemProxy(value)

    const success = await kernelStore.applyProxySettings()
    if (success) {
      message.success(t('notification.proxyModeChanged'))
    } else {
      message.error(getKernelFailureText(t('notification.proxyModeChangeFailed')))
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
      if (dialogReactive) dialogReactive.loading = true

      try {
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

    if (isUnixPlatform.value) {
      const code = parseSudoCode(getKernelFailureText(''))
      if (code === 'required' || code === 'invalid') {
        message.error(
          code === 'invalid' ? t('home.sudoPassword.invalid') : t('home.sudoPassword.required'),
        )

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
      await checkAdmin()

      if (isAdmin.value) {
        await enableTunWithKernelRestart()
      } else {
        await confirmTunSwitch()
      }
    } else if (isUnixPlatform.value) {
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
    try {
      modeSwitchPending.value = true
      await appStore.toggleTun(false)

      const applied = await kernelStore.applyProxySettings()
      if (!applied) {
        await appStore.toggleTun(true)
        message.error(t('notification.proxyModeChangeFailed'))
        return
      }

      const success = await kernelStore.restartKernel()
      if (success) {
        message.success(t('notification.proxyModeChanged'))
      } else {
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
      message.error(getKernelFailureText(t('home.restartFailed')))
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

const requestRestartAsAdmin = async () => {
  await systemService.restartAsAdmin()
}

const prepareTunModeWithAdminRestart = async () => {
  try {
    await appStore.toggleTun(true)
    const applied = await kernelStore.applyProxySettings()
    if (!applied) {
      await appStore.toggleTun(false)
      message.error(t('notification.proxyModeChangeFailed'))
      return false
    }
    await appStore.saveToBackend()

    if (appStore.isRunning) {
      await kernelStore.stopKernel()
    }

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
    const result = await kernelService.switchNodeProxyMode(mode as 'global' | 'rule')
    await syncCurrentNodeProxyMode()

    if (result.includes('重启后生效')) {
      message.warning(result)
      return
    }

    message.success(t('home.nodeModeChangeSuccess'))
  } catch (error) {
    message.error(t('home.nodeModeChangeFailed'))
  }
}

const checkAdmin = async () => {
  try {
    isAdmin.value = await systemService.checkAdmin()
  } catch (error) {
    isAdmin.value = false
  }
}

onMounted(async () => {
  try {
    const raw = await systemService.getPlatformInfo()
    platform.value = raw === 'windows' || raw === 'linux' || raw === 'macos' ? raw : 'unknown'
  } catch (error) {
    platform.value = 'unknown'
  }
  checkAdmin()
  await kernelStore.initializeStore()
  await proxyStore.fetchProxies().catch(() => undefined)
  await syncCurrentNodeProxyMode()
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

.home-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.status-hero {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 20px;
  border-radius: 14px;
  border: 1px solid var(--panel-border);
  background: var(--panel-bg);
}

.status-indicator {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--text-tertiary);
  flex-shrink: 0;
}

.status-hero.running {
  border-color: rgba(16, 185, 129, 0.2);
  background: rgba(16, 185, 129, 0.04);
}

.status-hero.running .status-indicator {
  background: #10b981;
  box-shadow: 0 0 12px rgba(16, 185, 129, 0.5);
}

.status-hero.running .status-label {
  color: #10b981;
}

.status-hero.pending,
.status-hero.disconnected {
  border-color: rgba(245, 158, 11, 0.2);
}

.status-hero.pending .status-indicator,
.status-hero.disconnected .status-indicator {
  background: #f59e0b;
  box-shadow: 0 0 8px rgba(245, 158, 11, 0.4);
}

.status-hero.pending .status-label,
.status-hero.disconnected .status-label {
  color: #f59e0b;
}

.status-hero.stopped,
.status-hero.failed {
  border-color: rgba(239, 68, 68, 0.2);
}

.status-hero.stopped .status-indicator,
.status-hero.failed .status-indicator {
  background: #ef4444;
  box-shadow: 0 0 8px rgba(239, 68, 68, 0.4);
}

.status-hero.stopped .status-label,
.status-hero.failed .status-label {
  color: #ef4444;
}

.status-hero.crashed {
  border-color: rgba(249, 115, 22, 0.2);
}

.status-hero.crashed .status-indicator {
  background: #f97316;
  box-shadow: 0 0 8px rgba(249, 115, 22, 0.4);
}

.status-hero.crashed .status-label {
  color: #f97316;
}

.status-label {
  font-size: 15px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.action-button {
  border-radius: 12px;
  font-weight: 600;
}

.diagnosis-alert {
  margin-bottom: 12px;
}

.diagnosis-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.diagnosis-meta {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.diagnosis-detail {
  white-space: pre-wrap;
  word-break: break-word;
}

.diagnosis-actions {
  margin: 0;
  padding-left: 18px;
}

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

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.card-header-icon {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.card-header-icon.port-icon {
  background: rgba(14, 165, 233, 0.1);
  color: #0ea5e9;
}

.card-header-icon.flow-icon {
  background: rgba(168, 85, 247, 0.1);
  color: #a855f7;
}

.card-header-icon.node-icon {
  background: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.card-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin: 0;
  flex: 1;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.proxy-port-card {
  padding: 14px;
  border-radius: 14px;
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.port-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-radius: 10px;
  background: var(--bg-tertiary);
}

.port-protocol {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-tertiary);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.port-address {
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', monospace;
  font-size: 13px;
  color: var(--text-primary);
}

.mode-cards {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mode-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px;
  border-radius: 14px;
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  cursor: pointer;
  transition: all 0.2s ease;
}

.mode-card:hover {
  border-color: var(--border-hover);
  background: var(--bg-secondary);
}

.mode-card.active {
  border-color: var(--primary-color);
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.04), rgba(99, 102, 241, 0.01));
}

.mode-card-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.mode-card-icon.active {
  background: var(--primary-color);
  color: white;
  box-shadow: 0 4px 14px rgba(99, 102, 241, 0.3);
}

.mode-card-info {
  flex: 1;
  min-width: 0;
}

.mode-card-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.mode-card-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 1px;
}

.radio-dot {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  border: 2px solid var(--border-color);
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.radio-dot.active {
  border-color: var(--primary-color);
  background: var(--primary-color);
  box-shadow: inset 0 0 0 3px var(--panel-bg);
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
  margin-top: 20px;
  height: 200px;
  background: var(--glass-bg);
  border-radius: 14px;
  border: 1px solid var(--glass-border);
  overflow: hidden;
  padding: 14px;
}
</style>
