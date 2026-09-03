<template>
  <div class="page-shell home-page">
    <!-- Hero 主控卡：运行状态 + 实时速率 + 操作 -->
    <section class="hero-card" :class="statusClass">
      <div class="hero-glow"></div>
      <div class="hero-inner">
        <div class="hero-status">
          <div class="hero-status-dot"></div>
          <div class="hero-info">
            <h2 class="hero-title">{{ statusTitle }}</h2>
            <p class="hero-subtitle">{{ statusDescription }}</p>
          </div>
        </div>

        <div class="hero-speeds">
          <div class="speed-metric">
            <span class="speed-icon up">
              <n-icon :size="15"><ArrowUpOutline /></n-icon>
            </span>
            <div class="speed-data">
              <span class="speed-value">{{ formatSpeed(trafficStore.traffic.up) }}</span>
              <span class="speed-label">{{ t('home.traffic.up') }}</span>
            </div>
          </div>
          <div class="speed-divider"></div>
          <div class="speed-metric">
            <span class="speed-icon down">
              <n-icon :size="15"><ArrowDownOutline /></n-icon>
            </span>
            <div class="speed-data">
              <span class="speed-value">{{ formatSpeed(trafficStore.traffic.down) }}</span>
              <span class="speed-label">{{ t('home.traffic.down') }}</span>
            </div>
          </div>
        </div>

        <div class="hero-actions">
          <n-button
            :type="kernelRunning ? 'error' : 'primary'"
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
              <n-button secondary type="warning" @click="restartAsAdmin">
                <template #icon>
                  <n-icon><ShieldCheckmarkOutline /></n-icon>
                </template>
              </n-button>
            </template>
            {{ t('home.restartAsAdmin') }}
          </n-tooltip>
        </div>
      </div>
    </section>

    <!-- 启动诊断告警 -->
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

    <!-- 控制台：一键测速 / 系统代理 / TUN / 节点模式 -->
    <div class="control-grid">
      <button type="button" class="control-card" :disabled="quickTesting" @click="quickTestAll">
        <span class="control-icon purple">
          <n-icon :size="20" :class="{ spinning: quickTesting }"><SpeedometerOutline /></n-icon>
        </span>
        <span class="control-body">
          <span class="control-name">{{ t('home.quick.testAll') }}</span>
          <span class="control-meta">{{
            quickTesting ? t('proxy.testing') : t('home.quick.testAllDesc')
          }}</span>
        </span>
      </button>

      <div class="control-card" :class="{ active: systemProxyEnabled }">
        <span class="control-icon" :class="systemProxyEnabled ? 'green' : 'gray'">
          <n-icon :size="20"><GlobeOutline /></n-icon>
        </span>
        <span class="control-body">
          <span class="control-name">{{ t('home.proxyMode.system') }}</span>
          <code class="control-meta mono">{{ proxyAddress }}</code>
        </span>
        <n-switch
          :value="systemProxyEnabled"
          size="small"
          :disabled="modeSwitchPending"
          @update:value="(v: boolean) => toggleSystemProxy(v)"
        />
      </div>

      <div class="control-card" :class="{ active: tunProxyEnabled }">
        <span class="control-icon" :class="tunProxyEnabled ? 'green' : 'gray'">
          <n-icon :size="20"><FlashOutline /></n-icon>
        </span>
        <span class="control-body">
          <span class="control-name">{{ t('home.proxyMode.tun') }}</span>
          <span class="control-meta" :title="t('home.proxyMode.tunTip')">
            {{ t('home.proxyMode.tunTip') }}
          </span>
        </span>
        <n-switch
          :value="tunProxyEnabled"
          size="small"
          :disabled="modeSwitchPending"
          @update:value="(v: boolean) => toggleTunProxy(v)"
        />
      </div>

      <div class="control-card mode-card">
        <span class="control-icon blue">
          <n-icon :size="20"><RadioOutline /></n-icon>
        </span>
        <div class="mode-content">
          <span class="control-name">{{ t('home.proxyHeader.nodeMode') }}</span>
          <div class="mode-chips">
            <button
              v-for="mode in nodeProxyModes"
              :key="mode.value"
              type="button"
              class="mode-chip"
              :class="{ active: currentNodeProxyMode === mode.value }"
              @click="handleNodeProxyModeChange(mode.value)"
            >
              <n-icon :size="14"><component :is="mode.icon" /></n-icon>
              <span>{{ t(mode.nameKey) }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 主网格：流量监控 + 运行信息 -->
    <div class="main-grid">
      <SectionCard flush class="chart-panel" :title="t('home.traffic.title')">
        <div class="chart-inner">
          <TrafficChart
            :upload-speed="trafficStore.traffic.up"
            :download-speed="trafficStore.traffic.down"
          />
        </div>
      </SectionCard>

      <SectionCard class="info-panel">
        <div class="info-rows">
          <div class="info-row">
            <span class="info-label">{{ t('home.quick.proxyAddr') }}</span>
            <span class="info-value-wrap">
              <code class="info-value">{{ proxyAddress }}</code>
              <n-button size="tiny" quaternary @click="showPortModal = true">
                {{ t('common.edit') }}
              </n-button>
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">{{ t('nav.connections') }}</span>
            <code class="info-value">{{ connectionStore.connections.length }}</code>
          </div>
          <div class="info-row">
            <span class="info-label">{{ t('home.traffic.uploadTotal') }}</span>
            <code class="info-value">{{ formatBytes(trafficStore.traffic.totalUp) }}</code>
          </div>
          <div class="info-row">
            <span class="info-label">{{ t('home.traffic.downloadTotal') }}</span>
            <code class="info-value">{{ formatBytes(trafficStore.traffic.totalDown) }}</code>
          </div>
          <div class="info-row">
            <span class="info-label">{{ t('home.memory') }}</span>
            <code class="info-value">{{ formatBytes(connectionStore.memory.inuse) }}</code>
          </div>
        </div>
      </SectionCard>
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
  SpeedometerOutline,
  ArrowUpOutline,
  ArrowDownOutline,
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
import SectionCard from '@/components/common/SectionCard.vue'
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
const quickTesting = ref(false)

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

// 状态描述（提供比状态名更有价值的说明）
const statusDescription = computed(() => {
  switch (statusState.value) {
    case 'running':
      return t('home.statusDescriptions.runningDesc')
    case 'disconnected':
      return t('home.statusDescriptions.disconnectedDesc')
    case 'starting':
      return t('home.kernelStatusDescriptions.startingDesc')
    case 'stopping':
      return t('home.kernelStatusDescriptions.stoppingDesc')
    case 'stopped':
    case 'failed':
    case 'crashed':
      return t('home.kernelStatusDescriptions.stoppedDesc')
    default:
      return t('home.kernelStatusDescriptions.stoppedDesc')
  }
})

const systemProxyEnabled = computed(() => appStore.systemProxyEnabled)
const tunProxyEnabled = computed(() => appStore.tunEnabled)
const proxyAddress = computed(() => `127.0.0.1:${appStore.proxyPort}`)

const nodeProxyModes = [
  {
    value: 'global',
    nameKey: 'home.nodeMode.global',
    icon: GlobeOutline,
  },
  {
    value: 'rule',
    nameKey: 'home.nodeMode.rule',
    icon: RadioOutline,
  },
]

const getKernelFailureText = (fallback: string) =>
  kernelStore.startupDiagnosisSummary || kernelStore.lastError || fallback

const syncCurrentNodeProxyMode = async () => {
  try {
    const status = await proxyService.getClashModeStatus()
    const mode = status.currentMode
    if (mode === 'global' || mode === 'rule' || mode === 'direct') {
      currentNodeProxyMode.value = mode
    }
  } catch {
  }
}

// 一键测速
const quickTestAll = async () => {
  if (quickTesting.value) return
  try {
    quickTesting.value = true
    await proxyStore.fetchProxies()
    await proxyStore.testAllGroups()
    message.success(t('proxy.batchTestComplete'))
  } catch {
    message.error(t('proxy.testErrorMessage'))
  } finally {
    quickTesting.value = false
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
  } catch {
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
  } catch {
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
        message.error(t('notification.proxyModeChangeFailed'))
      }
    } catch {
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
  } catch {
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
  } catch {
    message.error(t('home.nodeModeChangeFailed'))
  }
}

const checkAdmin = async () => {
  try {
    isAdmin.value = await systemService.checkAdmin()
  } catch {
    isAdmin.value = false
  }
}

onMounted(async () => {
  try {
    const raw = await systemService.getPlatformInfo()
    platform.value = raw === 'windows' || raw === 'linux' || raw === 'macos' ? raw : 'unknown'
  } catch {
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
  max-width: var(--content-max-width, 1440px);
  margin: 0 auto;
}

/* ============ Hero 主控卡 ============ */
.hero-card {
  position: relative;
  border-radius: var(--radius-xl);
  padding: clamp(18px, 2vw, 24px);
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  box-shadow: var(--shadow-md);
  overflow: hidden;
}

.hero-glow {
  position: absolute;
  top: -60px;
  right: -60px;
  width: 220px;
  height: 220px;
  border-radius: 50%;
  opacity: 0;
  transition: opacity var(--transition-slow);
  pointer-events: none;
}

.hero-card.running .hero-glow {
  background: radial-gradient(circle, var(--green-500-soft), transparent 70%);
  opacity: 1;
}

.hero-card.failed .hero-glow,
.hero-card.stopped .hero-glow {
  background: radial-gradient(circle, var(--red-500-soft), transparent 70%);
  opacity: 1;
}

.hero-card.pending .hero-glow,
.hero-card.disconnected .hero-glow {
  background: radial-gradient(circle, var(--amber-500-soft), transparent 70%);
  opacity: 1;
}

.hero-inner {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-5);
  flex-wrap: wrap;
}

.hero-status {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  min-width: 0;
}

.hero-status-dot {
  width: 14px;
  height: 14px;
  border-radius: var(--radius-pill);
  background: var(--text-tertiary);
  flex-shrink: 0;
  transition: all var(--transition-base);
}

.hero-card.running .hero-status-dot {
  background: var(--success-color);
  box-shadow: 0 0 16px var(--success-color), 0 0 4px var(--success-color);
  animation: pulse-green 2s ease-in-out infinite;
}

.hero-card.pending .hero-status-dot,
.hero-card.disconnected .hero-status-dot {
  background: var(--warning-color);
  box-shadow: 0 0 12px var(--warning-color);
}

.hero-card.stopped .hero-status-dot,
.hero-card.failed .hero-status-dot {
  background: var(--error-color);
  box-shadow: 0 0 12px var(--error-color);
}

.hero-card.crashed .hero-status-dot {
  background: var(--orange-500);
  box-shadow: 0 0 12px var(--orange-500);
}

@keyframes pulse-green {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.55;
  }
}

.hero-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.hero-title {
  margin: 0;
  font-size: var(--text-xl);
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.hero-card.running .hero-title {
  color: var(--success-color);
}

.hero-card.failed .hero-title,
.hero-card.stopped .hero-title {
  color: var(--error-color);
}

.hero-subtitle {
  margin: 0;
  font-size: var(--text-sm);
  color: var(--text-tertiary);
}

/* 实时速率 */
.hero-speeds {
  display: flex;
  align-items: center;
  gap: var(--space-5);
}

.speed-metric {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.speed-icon {
  width: 30px;
  height: 30px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.speed-icon.up {
  background: var(--green-500-soft);
  color: var(--success-color);
}

.speed-icon.down {
  background: var(--primary-soft);
  color: var(--primary-color);
}

.speed-data {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.speed-value {
  font-family: var(--font-mono);
  font-size: var(--text-lg);
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.1;
  font-variant-numeric: tabular-nums;
}

.speed-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.speed-divider {
  width: 1px;
  height: 30px;
  background: var(--panel-border);
}

.hero-actions {
  display: flex;
  gap: var(--space-2);
  flex-shrink: 0;
  margin-left: auto;
}

/* ============ 诊断告警 ============ */
.diagnosis-alert {
  border-radius: var(--radius-lg);
}

.diagnosis-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.diagnosis-meta {
  display: flex;
  gap: var(--space-2);
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

/* ============ 控制台卡片 ============ */
.control-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: var(--space-3);
}

.control-card {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-lg);
  background: var(--panel-bg);
  box-shadow: var(--shadow-xs);
  transition:
    transform var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
  text-align: left;
  min-width: 0;
}

button.control-card {
  font: inherit;
  color: inherit;
  cursor: pointer;
}

.control-card:hover:not(:disabled) {
  transform: translateY(-2px);
  border-color: var(--border-hover);
  box-shadow: var(--shadow-md);
}

.control-card:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.control-card:focus-visible,
.mode-chip:focus-visible {
  outline: none;
  box-shadow: var(--shadow-focus);
}

.control-card.active {
  border-color: var(--primary-color);
  box-shadow: 0 4px 14px var(--primary-soft-strong);
}

.control-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  flex-shrink: 0;
}

.control-icon.purple {
  background: linear-gradient(135deg, var(--indigo-400), var(--indigo-600));
}

.control-icon.green {
  background: linear-gradient(135deg, var(--green-400), var(--green-600));
}

.control-icon.blue {
  background: linear-gradient(135deg, var(--blue-400), var(--blue-600));
}

.control-icon.gray {
  background: var(--bg-surface-2);
  color: var(--text-tertiary);
}

.control-icon .spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.control-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.control-name {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.control-meta {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.control-meta.mono {
  font-family: var(--font-mono);
}

/* 节点模式卡 */
.mode-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  flex: 1;
  min-width: 0;
}

.mode-chips {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.mode-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 12px;
  border: none;
  border-radius: var(--radius-pill);
  background: var(--bg-surface-2);
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.mode-chip:hover {
  color: var(--text-primary);
}

.mode-chip.active {
  background: var(--primary-color);
  color: var(--primary-contrast);
  box-shadow: 0 2px 8px var(--primary-soft-strong);
}

/* ============ 主网格：流量监控 + 运行信息 ============ */
.main-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 320px;
  gap: var(--space-4);
  min-height: 0;
}

.chart-inner {
  height: 240px;
  padding: var(--space-2) var(--space-4) var(--space-4);
}

/* 运行信息卡 */
.info-panel {
  display: flex;
  flex-direction: column;
}

.info-rows {
  display: flex;
  flex-direction: column;
  flex: 1;
  justify-content: center;
}

.info-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-2) 0;
}

.info-row + .info-row {
  border-top: 1px dashed var(--border-color);
}

.info-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.info-value-wrap {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  min-width: 0;
}

.info-value {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  font-weight: 600;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

@media (max-width: 1080px) {
  .main-grid {
    grid-template-columns: 1fr;
  }

  .hero-actions {
    margin-left: 0;
  }
}
</style>
