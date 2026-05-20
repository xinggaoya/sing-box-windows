<template>
  <div class="home-page">
    <div class="hero-section" :class="statusClass">
      <div class="hero-bg-glow"></div>
      <div class="hero-content">
        <div class="hero-row-top">
          <div class="hero-left">
            <div class="hero-status-dot"></div>
            <div class="hero-info">
              <div class="hero-title-row">
                <h2 class="hero-title">{{ statusTitle }}</h2>
                <span class="hero-speed-item">
                  <span class="meta-arrow up">↑</span>
                  {{ formatSpeed(trafficStore.traffic.up) }}
                </span>
                <span class="hero-speed-item">
                  <span class="meta-arrow down">↓</span>
                  {{ formatSpeed(trafficStore.traffic.down) }}
                </span>
              </div>
            </div>
          </div>
          <div class="hero-actions">
            <n-button
              :type="kernelRunning ? 'error' : 'primary'"
              size="large"
              round
              :loading="kernelLoading"
              @click="restartKernel"
            >
              <template #icon>
                <n-icon><PowerOutline /></n-icon>
              </template>
              {{ t('home.restart') }}
            </n-button>
            <n-tooltip v-if="isWindowsPlatform && !isAdmin" trigger="hover">
              <template #trigger>
                <n-button
                  size="large"
                  round
                  secondary
                  type="warning"
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

        <div class="hero-row-stats" v-if="kernelRunning">
          <div class="hero-stat">
            <span class="hero-stat-label">HTTP</span>
            <code class="hero-stat-value">{{ proxyAddress }}</code>
          </div>
          <div class="hero-stat-sep"></div>
          <div class="hero-stat">
            <span class="hero-stat-label">SOCKS5</span>
            <code class="hero-stat-value">{{ proxyAddress }}</code>
          </div>
          <div class="hero-stat-sep"></div>
          <div class="hero-stat">
            <span class="hero-stat-label">{{ t('nav.connections') }}</span>
            <code class="hero-stat-value">{{ connectionStore.connections.length }}</code>
          </div>
          <div class="hero-stat-sep"></div>
          <div class="hero-stat">
            <span class="hero-stat-label">{{ t('proxy.title') }}</span>
            <code class="hero-stat-value">{{ currentNodeProxyMode === 'global' ? t('home.nodeMode.global') : t('home.nodeMode.rule') }}</code>
          </div>
          <div class="hero-stat-sep"></div>
          <div class="hero-stat">
            <span class="hero-stat-label">{{ t('home.traffic.total') }}</span>
            <code class="hero-stat-value">{{ formatBytes(trafficStore.traffic.totalUp + trafficStore.traffic.totalDown) }}</code>
          </div>
        </div>
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

    <div class="main-grid">
      <div class="chart-panel">
        <TrafficChart
          :upload-speed="trafficStore.traffic.up"
          :download-speed="trafficStore.traffic.down"
        />
      </div>

      <div class="side-panels">
        <div class="mode-panel">
          <div class="panel-header">
            <span class="panel-title">{{ t('home.proxyHeader.flowMode') }}</span>
            <n-button size="tiny" quaternary @click="showPortModal = true">
              {{ t('common.edit') }}
            </n-button>
          </div>
          <div class="toggle-list">
            <div class="toggle-item" :class="{ active: systemProxyEnabled }">
              <div class="toggle-icon">
                <n-icon :size="18"><GlobeOutline /></n-icon>
              </div>
              <div class="toggle-info">
                <span class="toggle-name">{{ t('home.proxyMode.system') }}</span>
                <code class="toggle-port">{{ proxyAddress }}</code>
              </div>
              <n-switch :value="systemProxyEnabled" size="small" :disabled="modeSwitchPending" @update:value="(v: boolean) => toggleSystemProxy(v)" />
            </div>
            <div class="toggle-item" :class="{ active: tunProxyEnabled }">
              <div class="toggle-icon">
                <n-icon :size="18"><FlashOutline /></n-icon>
              </div>
              <div class="toggle-info">
                <span class="toggle-name">{{ t('home.proxyMode.tun') }}</span>
                <span class="toggle-desc">{{ t('home.proxyMode.tunTip') }}</span>
              </div>
              <n-switch :value="tunProxyEnabled" size="small" :disabled="modeSwitchPending" @update:value="(v: boolean) => toggleTunProxy(v)" />
            </div>
          </div>
        </div>

        <div class="proxy-mode-panel">
          <div class="panel-header">
            <span class="panel-title">{{ t('home.proxyHeader.nodeMode') }}</span>
          </div>
          <div class="mode-chips">
            <div
              v-for="mode in nodeProxyModes"
              :key="mode.value"
              class="mode-chip"
              :class="{ active: currentNodeProxyMode === mode.value }"
              @click="handleNodeProxyModeChange(mode.value)"
            >
              <n-icon :size="15"><component :is="mode.icon" /></n-icon>
              <span>{{ t(mode.nameKey) }}</span>
            </div>
          </div>
        </div>

        <div class="traffic-info">
          <div class="traffic-row">
            <span class="traffic-label">
              <span class="traffic-dot upload"></span>
              {{ t('home.traffic.up') }}
            </span>
            <span class="traffic-val">{{ formatBytes(trafficStore.traffic.totalUp) }}</span>
          </div>
          <div class="traffic-divider"></div>
          <div class="traffic-row">
            <span class="traffic-label">
              <span class="traffic-dot download"></span>
              {{ t('home.traffic.down') }}
            </span>
            <span class="traffic-val">{{ formatBytes(trafficStore.traffic.totalDown) }}</span>
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
  GlobeOutline,
  FlashOutline,
  RadioOutline,
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
import { kernelService } from '@/services/kernel-service'
import { proxyService } from '@/services/proxy-service'
import { sudoService } from '@/services/sudo-service'
import { systemService } from '@/services/system-service'
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
  } catch {
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
.home-page {
  padding: var(--layout-page-padding-y, 16px) var(--layout-page-padding-x, 24px);
  max-width: var(--layout-page-max-width, 1200px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hero-section {
  position: relative;
  border-radius: 20px;
  padding: 24px 28px;
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  overflow: hidden;
}

.hero-bg-glow {
  position: absolute;
  top: -40px;
  right: -40px;
  width: 200px;
  height: 200px;
  border-radius: 50%;
  opacity: 0;
  transition: opacity 0.5s ease;
  pointer-events: none;
}

.hero-section.running .hero-bg-glow {
  background: radial-gradient(circle, rgba(16, 185, 129, 0.12), transparent 70%);
  opacity: 1;
}

.hero-section.failed .hero-bg-glow,
.hero-section.stopped .hero-bg-glow {
  background: radial-gradient(circle, rgba(239, 68, 68, 0.08), transparent 70%);
  opacity: 1;
}

.hero-section.pending .hero-bg-glow,
.hero-section.disconnected .hero-bg-glow {
  background: radial-gradient(circle, rgba(245, 158, 11, 0.08), transparent 70%);
  opacity: 1;
}

.hero-content {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.hero-row-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
}

.hero-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.hero-status-dot {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--text-tertiary);
  flex-shrink: 0;
  transition: all 0.4s ease;
}

.hero-section.running .hero-status-dot {
  background: #10b981;
  box-shadow: 0 0 16px rgba(16, 185, 129, 0.5), 0 0 4px rgba(16, 185, 129, 0.8);
  animation: pulse-green 2s ease-in-out infinite;
}

.hero-section.pending .hero-status-dot,
.hero-section.disconnected .hero-status-dot {
  background: #f59e0b;
  box-shadow: 0 0 12px rgba(245, 158, 11, 0.4);
}

.hero-section.stopped .hero-status-dot,
.hero-section.failed .hero-status-dot {
  background: #ef4444;
  box-shadow: 0 0 12px rgba(239, 68, 68, 0.4);
}

.hero-section.crashed .hero-status-dot {
  background: #f97316;
  box-shadow: 0 0 12px rgba(249, 115, 22, 0.4);
}

@keyframes pulse-green {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.hero-info {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.hero-title-row {
  display: flex;
  align-items: baseline;
  gap: 16px;
}

.hero-title {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.hero-section.running .hero-title {
  color: #10b981;
}

.hero-section.failed .hero-title,
.hero-section.stopped .hero-title {
  color: #ef4444;
}

.hero-speed-item {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

.meta-arrow {
  font-size: 11px;
  font-weight: 800;
}

.meta-arrow.up {
  color: #10b981;
}

.meta-arrow.down {
  color: var(--primary-color);
}

.meta-dot {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: var(--text-tertiary);
}

.hero-actions {
  display: flex;
  gap: 10px;
  flex-shrink: 0;
}

.hero-row-stats {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 10px 14px;
  border-radius: 10px;
  background: var(--bg-tertiary);
  flex-wrap: wrap;
}

.hero-stat {
  display: flex;
  align-items: center;
  gap: 6px;
}

.hero-stat-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.hero-stat-value {
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', monospace;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.hero-stat-sep {
  width: 1px;
  height: 14px;
  background: var(--border-color);
  flex-shrink: 0;
}

.diagnosis-alert {
  border-radius: 14px;
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

.main-grid {
  display: grid;
  grid-template-columns: 1fr 320px;
  gap: 16px;
  min-height: 0;
}

.chart-panel {
  min-height: 280px;
  border-radius: 16px;
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  overflow: hidden;
  padding: 14px;
}

.side-panels {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.mode-panel,
.proxy-mode-panel {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  padding: 14px 16px;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.panel-title {
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-tertiary);
}

.toggle-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toggle-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 10px;
  background: var(--bg-tertiary);
  transition: all 0.2s ease;
}

.toggle-item.active {
  background: rgba(99, 102, 241, 0.06);
}

.toggle-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  color: var(--text-secondary);
  flex-shrink: 0;
}

.toggle-item.active .toggle-icon {
  background: var(--primary-color);
  color: white;
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.3);
}

.toggle-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.toggle-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.toggle-desc,
.toggle-port {
  font-size: 11px;
  color: var(--text-tertiary);
}

.toggle-port {
  font-family: 'SFMono-Regular', Consolas, monospace;
}

.mode-chips {
  display: flex;
  gap: 8px;
}

.mode-chip {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: 8px;
  background: var(--bg-tertiary);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.mode-chip:hover {
  color: var(--text-primary);
}

.mode-chip.active {
  background: var(--primary-color);
  color: white;
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.3);
}

.traffic-info {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  padding: 10px 16px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.traffic-divider {
  width: 1px;
  height: 14px;
  background: var(--border-color);
  flex-shrink: 0;
}

.traffic-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.traffic-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}

.traffic-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.traffic-dot.upload {
  background: #10b981;
}

.traffic-dot.download {
  background: var(--primary-color);
}

.traffic-val {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

@media (max-width: 900px) {
  .main-grid {
    grid-template-columns: 1fr;
  }

  .chart-panel {
    min-height: 200px;
  }

  .hero-content {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
