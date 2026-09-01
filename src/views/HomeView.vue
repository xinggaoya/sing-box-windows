<template>
  <div class="page-shell home-page">
    <!-- Hero 状态卡（精简：状态 + 重启，不再重复速度与统计） -->
    <section class="hero-card" :class="statusClass">
      <div class="hero-glow"></div>
      <div class="hero-inner">
        <div class="hero-top">
          <div class="hero-left">
            <div class="hero-status-dot"></div>
            <div class="hero-info">
              <h2 class="hero-title">{{ statusTitle }}</h2>
              <p class="hero-subtitle">{{ statusDescription }}</p>
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

    <!-- 快捷操作区（常用功能一键直达） -->
    <div class="quick-actions">
      <button class="quick-btn" :disabled="quickTesting" @click="quickTestAll">
        <div class="quick-icon purple">
          <n-icon :size="20"><SpeedometerOutline /></n-icon>
        </div>
        <span class="quick-label">{{ t('home.quick.testAll') }}</span>
      </button>
      <button
        class="quick-btn"
        :class="{ on: systemProxyEnabled }"
        :disabled="modeSwitchPending"
        @click="toggleSystemProxy(!systemProxyEnabled)"
      >
        <div class="quick-icon" :class="systemProxyEnabled ? 'green' : 'gray'">
          <n-icon :size="20"><GlobeOutline /></n-icon>
        </div>
        <span class="quick-label">{{ t('home.proxyMode.system') }}</span>
        <span class="quick-state" :class="systemProxyEnabled ? 'on' : 'off'">
          {{ systemProxyEnabled ? t('common.enabled') : t('common.disabled') }}
        </span>
      </button>
      <button
        class="quick-btn"
        :class="{ on: tunProxyEnabled }"
        :disabled="modeSwitchPending"
        @click="toggleTunProxy(!tunProxyEnabled)"
      >
        <div class="quick-icon" :class="tunProxyEnabled ? 'green' : 'gray'">
          <n-icon :size="20"><FlashOutline /></n-icon>
        </div>
        <span class="quick-label">{{ t('home.proxyMode.tun') }}</span>
        <span class="quick-state" :class="tunProxyEnabled ? 'on' : 'off'">
          {{ tunProxyEnabled ? t('common.enabled') : t('common.disabled') }}
        </span>
      </button>
      <button class="quick-btn" @click="cycleNodeProxyMode">
        <div class="quick-icon blue">
          <n-icon :size="20"><RadioOutline /></n-icon>
        </div>
        <span class="quick-label">{{ t('home.quick.nodeMode') }}</span>
        <span class="quick-state">{{
          currentNodeProxyMode === 'global'
            ? t('home.nodeMode.global')
            : t('home.nodeMode.rule')
        }}</span>
      </button>
    </div>

    <!-- 主网格：流量图 + 运行信息 -->
    <div class="main-grid">
      <SectionCard flush class="chart-panel">
        <div class="chart-inner">
          <TrafficChart
            :upload-speed="trafficStore.traffic.up"
            :download-speed="trafficStore.traffic.down"
          />
        </div>
      </SectionCard>

      <!-- 运行信息卡（精简不重复：代理地址 / 连接数 / 总流量 / 内存） -->
      <SectionCard class="info-panel">
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">{{ t('home.quick.proxyAddr') }}</span>
            <code class="info-value">{{ proxyAddress }}</code>
          </div>
          <div class="info-item">
            <span class="info-label">{{ t('nav.connections') }}</span>
            <code class="info-value">{{ connectionStore.connections.length }}</code>
          </div>
          <div class="info-item">
            <span class="info-label">{{ t('home.traffic.total') }}</span>
            <code class="info-value">{{
              formatBytes(trafficStore.traffic.totalUp + trafficStore.traffic.totalDown)
            }}</code>
          </div>
          <div class="info-item">
            <span class="info-label">{{ t('home.memory') }}</span>
            <code class="info-value">{{ formatBytes(connectionStore.memory.inuse) }}</code>
          </div>
        </div>
      </SectionCard>
    </div>

    <!-- 代理模式详细开关区 -->
    <div class="bottom-grid">
      <SectionCard>
        <template #actions>
          <n-button size="tiny" quaternary @click="showPortModal = true">
            {{ t('common.edit') }}
          </n-button>
        </template>
        <div class="toggle-list">
          <div class="toggle-item" :class="{ active: systemProxyEnabled }">
            <div class="toggle-icon">
              <n-icon :size="18"><GlobeOutline /></n-icon>
            </div>
            <div class="toggle-info">
              <span class="toggle-name">{{ t('home.proxyMode.system') }}</span>
              <code class="toggle-port">{{ proxyAddress }}</code>
            </div>
            <n-switch
              :value="systemProxyEnabled"
              size="small"
              :disabled="modeSwitchPending"
              @update:value="(v: boolean) => toggleSystemProxy(v)"
            />
          </div>
          <div class="toggle-item" :class="{ active: tunProxyEnabled }">
            <div class="toggle-icon">
              <n-icon :size="18"><FlashOutline /></n-icon>
            </div>
            <div class="toggle-info">
              <span class="toggle-name">{{ t('home.proxyMode.tun') }}</span>
              <span class="toggle-desc">{{ t('home.proxyMode.tunTip') }}</span>
            </div>
            <n-switch
              :value="tunProxyEnabled"
              size="small"
              :disabled="modeSwitchPending"
              @update:value="(v: boolean) => toggleTunProxy(v)"
            />
          </div>
        </div>
      </SectionCard>

      <SectionCard>
        <div class="mode-chips-wrap">
          <div class="mode-chips-title">{{ t('home.proxyHeader.nodeMode') }}</div>
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
import { formatBytes } from '@/utils'

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

// 状态描述（替代重复的统计条，提供更有价值的状态说明）
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
    const status = await proxyService.getClashModeStatus()
    const mode = status.currentMode
    if (mode === 'global' || mode === 'rule' || mode === 'direct') {
      currentNodeProxyMode.value = mode
    }
  } catch {
  }
}

// 快捷操作：一键全部测速
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

// 快捷操作：循环切换节点模式 global <-> rule
const cycleNodeProxyMode = async () => {
  const next = currentNodeProxyMode.value === 'global' ? 'rule' : 'global'
  await handleNodeProxyModeChange(next)
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

/* ============ Hero 卡 ============ */
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
}

.hero-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  flex-wrap: wrap;
}

.hero-left {
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

.hero-actions {
  display: flex;
  gap: var(--space-2);
  flex-shrink: 0;
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

/* ============ 快捷操作区 ============ */
.quick-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: var(--space-3);
}

.quick-btn {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-lg);
  background: var(--panel-bg);
  cursor: pointer;
  transition:
    transform var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
  text-align: left;
}

.quick-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  border-color: var(--border-hover);
  box-shadow: var(--shadow-md);
}

.quick-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.quick-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}

.quick-icon.purple {
  background: linear-gradient(135deg, var(--indigo-400), var(--indigo-600));
}

.quick-icon.green {
  background: linear-gradient(135deg, var(--green-400), var(--green-600));
}

.quick-icon.blue {
  background: linear-gradient(135deg, var(--blue-400), var(--blue-600));
}

.quick-icon.gray {
  background: var(--bg-surface-2);
  color: var(--text-tertiary);
}

.quick-label {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.quick-state {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-weight: 500;
}

.quick-state.on {
  color: var(--success-color);
}

.quick-state.off {
  color: var(--text-tertiary);
}

/* ============ 主网格：流量图 + 运行信息 ============ */
.main-grid {
  display: grid;
  grid-template-columns: 1fr 300px;
  gap: var(--space-4);
  min-height: 0;
}

.chart-panel {
  min-height: 0;
}

.chart-inner {
  height: 220px;
  padding: var(--space-3);
}

/* 运行信息卡 */
.info-panel {
  display: flex;
  flex-direction: column;
}

.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-3);
  flex: 1;
  align-content: center;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  background: var(--bg-surface-2);
}

.info-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.info-value {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  font-weight: 600;
  color: var(--text-primary);
}

/* ============ 底部网格：代理开关 + 节点模式 ============ */
.bottom-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-4);
}

.toggle-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.toggle-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--bg-surface-2);
  transition: background var(--transition-fast);
}

.toggle-item.active {
  background: var(--primary-soft);
}

.toggle-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  flex-shrink: 0;
}

.toggle-item.active .toggle-icon {
  background: var(--primary-color);
  color: var(--primary-contrast);
  box-shadow: 0 2px 8px var(--primary-soft-strong);
}

.toggle-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.toggle-name {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.toggle-desc,
.toggle-port {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.toggle-port {
  font-family: var(--font-mono);
}

/* 节点模式 */
.mode-chips-wrap {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.mode-chips-title {
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.mode-chips {
  display: flex;
  gap: var(--space-2);
}

.mode-chip {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  background: var(--bg-surface-2);
  font-size: var(--text-sm);
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

@media (max-width: 960px) {
  .main-grid,
  .bottom-grid {
    grid-template-columns: 1fr;
  }
}
</style>
