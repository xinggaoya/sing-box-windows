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
        <ul v-if="kernelStore.startupDiagnosis.suggested_actions?.length" class="diagnosis-actions">
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
import { useHomeEnvironment, useNodeProxyMode } from '@/composables/useHomeDashboard'
import { useAppStore } from '@/stores'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
import { sudoService } from '@/services/sudo-service'
import { systemService } from '@/services/system-service'
import PortSettingsDialog from '@/components/common/PortSettingsDialog.vue'
import TrafficChart from '@/components/layout/TrafficChart.vue'
import SectionCard from '@/components/common/SectionCard.vue'
import { useKernelStatus } from '@/composables/useKernelStatus'
import { prepareTunAndRestartAsAdmin, useTunToggle } from '@/composables/useTunToggle'
import { useSudoStore } from '@/stores'
import { formatBytes, formatSpeed } from '@/utils'
import { extractKernelErrorMessage, resolveKernelFailureText } from '@/utils/kernel-errors'

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
// 运行环境探测（平台 / 管理员权限）与节点模式（Clash mode）逻辑已抽到 composables：
// 前者决定 TUN 可用性，后者决定 sing-box 内部路由策略，与代理开关是三件不同的事。
const { isAdmin, isWindowsPlatform, isUnixPlatform, detectPlatform, checkAdmin } =
  useHomeEnvironment()

const {
  currentNodeProxyMode,
  quickTesting,
  nodeProxyModes,
  proxyAddress,
  syncCurrentNodeProxyMode,
  handleNodeProxyModeChange,
  quickTestAll,
} = useNodeProxyMode()

/** 系统代理开关进行中：与 TUN 切换互斥，避免两条状态机并发改入站 */
const proxyModePending = ref(false)
/** 「以管理员身份重启」确认弹窗流程进行中 */
const tunConfirmPending = ref(false)
const showPortModal = ref(false)

// 平台无关的 TUN 切换状态机；平台提权与 sudo 密码交互通过回调注入。
const { pending: tunTogglePending, applyTunSwitch } = useTunToggle({
  // 用 getter 而非字面量：i18n 文案在 setup 时求值一次就会被固定在普通对象里，
  // 组件内切换语言后不会被重新求值，错误提示会停留在旧语言
  messages: {
    applyFailed: () => t('notification.proxyModeChangeFailed'),
    restartFailed: () => t('home.restartFailed'),
  },
  // Unix 下内核启动前需要 sudo 密码；缺失时先索要再继续
  ensurePrivilege: async (target) => {
    if (!target || !isUnixPlatform.value) return 'proceed'

    const status = await sudoService.getStatus()
    if (!status.supported) {
      message.error(t('home.sudoPassword.unsupported'))
      // 不阻断：交给核心流程，最终会失败并回滚
      return 'proceed'
    }
    if (!status.has_saved) {
      const ok = await sudoStore.requestPassword()
      if (!ok) {
        message.error(t('home.sudoPassword.required'))
      }
    }
    return 'proceed'
  },
  handleSudoFailure: async (code) => {
    message.error(
      code === 'invalid' ? t('home.sudoPassword.invalid') : t('home.sudoPassword.required'),
    )
    return sudoStore.requestPassword()
  },
})

// 切换期间同时禁用两处入口（模式选择器与 TUN 开关）
const modeSwitchPending = computed(() => tunTogglePending.value || proxyModePending.value)

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
const getKernelFailureText = (fallback: string) =>
  resolveKernelFailureText(kernelStore.startupDiagnosisSummary, kernelStore.lastError, fallback)

const toggleSystemProxy = async (value: boolean) => {
  if (modeSwitchPending.value) return

  try {
    proxyModePending.value = true
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
    proxyModePending.value = false
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
      tunConfirmPending.value = true
      if (dialogReactive) dialogReactive.loading = true

      try {
        const success = await prepareTunModeWithAdminRestart()
        finish(success)
        return success
      } finally {
        tunConfirmPending.value = false
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

const toggleTunProxy = async (value: boolean) => {
  if (modeSwitchPending.value) return

  if (value && isWindowsPlatform.value) {
    // Windows 需要管理员权限才能创建 TUN 网卡
    await checkAdmin()
    if (!isAdmin.value) {
      // 无权限：走「确认 + 保存配置 + 管理员重启」流程
      await confirmTunSwitch()
      return
    }
  }

  const outcome = await applyTunSwitch(value)
  switch (outcome.status) {
    case 'success':
      message.success(t('notification.proxyModeChanged'))
      break
    case 'failed':
      message.error(outcome.failure.text)
      break
    default:
      // 'skipped'（状态未变）/ 'escalated'（已转交管理员重启）无需提示
      break
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
    const details = extractKernelErrorMessage(error)
    message.error(details ? `${t('home.restartFailed')}：${details}` : t('home.restartFailed'))
  }
}

const requestRestartAsAdmin = async () => {
  await systemService.restartAsAdmin()
}

const prepareTunModeWithAdminRestart = async () => {
  const success = await prepareTunAndRestartAsAdmin()
  if (!success) {
    message.error(t('notification.proxyModeChangeFailed'))
  }
  return success
}

onMounted(async () => {
  await detectPlatform()
  checkAdmin()
  await kernelStore.initializeStore()
  await proxyStore.fetchProxies().catch(() => undefined)
  await syncCurrentNodeProxyMode()
})
</script>

<style scoped>
/* 页面样式已抽到 `@/assets/views/HomeView.css`，用 @import 而非 script import：
 * 这样外部文件里的规则仍会经过 SFC 的 scoped 变换（加上 data-v-* 属性），
 * 保持与内联 <style scoped> 完全一致的隔离语义。 */
@import '@/assets/views/HomeView.css';
</style>
