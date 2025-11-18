<template>
  <div class="home-shell">
    <section class="hero-panel">
      <div class="hero-main">
        <div class="hero-status">
          <div class="status-glow" :class="statusOrbClass">
            <div class="status-inner">
              <n-icon :size="28" :class="statusIconClass">
                <PowerOutline v-if="!appStore.isRunning" />
                <CheckmarkCircleOutline v-else-if="kernelStore.status.websocket_ready" />
                <TimeOutline v-else-if="isStarting" />
                <CloseCircleOutline v-else />
              </n-icon>
            </div>
          </div>
          <div class="hero-texts">
            <h2 class="hero-title">{{ getStatusTitle() }}</h2>
            <p class="hero-description">{{ getStatusSubtitle() }}</p>
            <div class="hero-tags">
              <n-tag
                :type="kernelStore.status.websocket_ready ? 'success' : 'error'"
                size="small"
                round
                :bordered="false"
              >
                <template #icon>
                  <n-icon size="14">
                    <WifiOutline v-if="kernelStore.status.websocket_ready" />
                    <CloseCircleOutline v-else />
                  </n-icon>
                </template>
                {{
                  kernelStore.status.websocket_ready
                    ? t('home.wsStatus.connected')
                    : t('home.wsStatus.disconnected')
                }}
              </n-tag>
              <n-tag :type="isAdmin ? 'success' : 'warning'" size="small" round :bordered="false">
                <template #icon>
                  <n-icon size="14">
                    <ShieldCheckmarkOutline />
                  </n-icon>
                </template>
                {{ isAdmin ? t('home.adminStatus.admin') : t('home.adminStatus.normal') }}
              </n-tag>
            </div>
          </div>
        </div>
        <div class="hero-actions">
          <div class="hero-secondary">
            <n-tooltip :show-arrow="false">
              <template #trigger>
                <n-button
                  tertiary
                  size="medium"
                  class="rounded-control"
                  @click="restartKernel"
                  :loading="isStarting || isStopping"
                >
                  <template #icon>
                    <n-icon :size="18">
                      <RefreshOutline />
                    </n-icon>
                  </template>
                  {{ t('home.restart') }}
                </n-button>
              </template>
              {{ t('home.restart') }}
            </n-tooltip>
            <n-tooltip :show-arrow="false" v-if="!isAdmin">
              <template #trigger>
                <n-button class="rounded-control" tertiary size="medium" @click="restartAsAdmin">
                  <template #icon>
                    <n-icon :size="18">
                      <ShieldCheckmarkOutline />
                    </n-icon>
                  </template>
                  {{ t('home.restartAsAdmin') }}
                </n-button>
              </template>
              {{ t('home.restartAsAdmin') }}
            </n-tooltip>
          </div>
        </div>
      </div>
    </section>

    <section class="content-grid">
      <n-grid :cols="24" :x-gap="16" :y-gap="16" responsive="screen">
        <n-grid-item :span="24" :s="24" :m="16" :l="16" :xl="16" :xxl="16">
          <n-card class="panel-card" :bordered="false">
            <div class="panel-head">
              <div>
                <p class="panel-eyebrow">{{ t('home.proxyHeader.flowMode') }}</p>
                <h3 class="panel-title">{{ getCurrentProxyModeName() }}</h3>
              </div>
              <n-tag
                :type="getProxyModeTagType(appStore.proxyMode)"
                size="small"
                round
                :bordered="false"
              >
                {{ getCurrentProxyModeName() }}
              </n-tag>
            </div>
            <div class="mode-toggle-list">
              <div class="mode-toggle">
                <div class="mode-toggle-info">
                  <div class="mode-icon">
                    <n-icon :size="24">
                      <component :is="proxyModes[0].icon" />
                    </n-icon>
                  </div>
                  <div class="mode-content">
                    <h4 class="mode-name">{{ t(proxyModes[0].nameKey) }}</h4>
                    <p class="mode-description">{{ t(proxyModes[0].tipKey) }}</p>
                  </div>
                </div>
                <n-switch
                  :value="systemProxyEnabled"
                  :loading="modeSwitchPending"
                  :disabled="modeSwitchPending"
                  size="large"
                  @update:value="toggleSystemProxy"
                />
              </div>
              <div class="mode-toggle">
                <div class="mode-toggle-info">
                  <div class="mode-icon">
                    <n-icon :size="24">
                      <component :is="proxyModes[1].icon" />
                    </n-icon>
                  </div>
                  <div class="mode-content">
                    <h4 class="mode-name">{{ t(proxyModes[1].nameKey) }}</h4>
                    <p class="mode-description">{{ t(proxyModes[1].tipKey) }}</p>
                  </div>
                </div>
                <n-switch
                  :value="tunProxyEnabled"
                  :loading="modeSwitchPending"
                  :disabled="modeSwitchPending"
                  size="large"
                  @update:value="toggleTunProxy"
                />
              </div>
            </div>
          </n-card>

          <n-card class="panel-card" :bordered="false">
            <div class="panel-head">
              <div>
                <p class="panel-eyebrow">{{ t('home.proxyHeader.nodeMode') }}</p>
                <h3 class="panel-title">{{ getCurrentNodeProxyModeName() }}</h3>
              </div>
              <n-tag
                :type="getNodeProxyModeTagType(currentNodeProxyMode)"
                size="small"
                round
                :bordered="false"
              >
                {{ getCurrentNodeProxyModeName() }}
              </n-tag>
            </div>
            <div class="mode-grid">
              <div
                v-for="mode in nodeProxyModes"
                :key="mode.value"
                class="mode-card"
                :class="{ 'mode-active': currentNodeProxyMode === mode.value }"
                @click="handleNodeProxyModeChange(mode.value)"
              >
                <div class="mode-icon">
                  <n-icon :size="24">
                    <component :is="mode.icon" />
                  </n-icon>
                </div>
                <div class="mode-content">
                  <h4 class="mode-name">{{ t(mode.nameKey) }}</h4>
                  <p class="mode-description">{{ t(mode.tipKey) }}</p>
                </div>
                <div class="mode-indicator">
                  <span></span>
                </div>
              </div>
            </div>
          </n-card>
        </n-grid-item>

        <n-grid-item :span="24" :s="24" :m="8" :l="8" :xl="8" :xxl="8">
          <n-card class="panel-card traffic-card" :bordered="false">
            <TrafficStatsCard
              :active-connections-count="String(connectionStore.connections.length)"
              :traffic-up="trafficStore.traffic.up"
              :traffic-down="trafficStore.traffic.down"
              :total-up="trafficStore.traffic.totalUp"
              :total-down="trafficStore.traffic.totalDown"
              :memory="connectionStore.memory.inuse"
              :is-route-active="true"
            />
          </n-card>
        </n-grid-item>
      </n-grid>
    </section>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  NCard,
  NButton,
  NIcon,
  NGrid,
  NGridItem,
  NTag,
  NTooltip,
  NSwitch,
  useDialog,
  useMessage,
} from 'naive-ui'
import {
  RefreshOutline,
  SettingsOutline,
  GlobeOutline,
  FlashOutline,
  ShieldCheckmarkOutline,
  PowerOutline,
  CheckmarkCircleOutline,
  TimeOutline,
  CloseCircleOutline,
  WifiOutline,
  RadioOutline,
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { kernelApi } from '@/services/tauri'
import TrafficStatsCard from '@/components/home/TrafficStatsCard.vue'
import type { ProxyMode } from '@/stores/app/AppStore'

defineOptions({
  name: 'HomeView',
})

const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()

// Store实例
const appStore = useAppStore()
const kernelStore = useKernelStore()
const trafficStore = useTrafficStore()
const connectionStore = useConnectionStore()
const themeStore = useThemeStore()

// 响应式状态
const isStarting = ref(false)
const isStopping = ref(false)
const isAdmin = ref(false)
const currentNodeProxyMode = ref('rule')
const modeSwitchPending = ref(false)

// 计算属性
const statusOrbClass = computed(() => {
  if (!appStore.isRunning) return 'orb-stopped'
  if (kernelStore.status.websocket_ready) return 'orb-connected'
  return 'orb-connecting'
})

const statusIconClass = computed(() => {
  if (!appStore.isRunning) return 'icon-stopped'
  if (kernelStore.status.websocket_ready) return 'icon-connected'
  return 'icon-connecting'
})

const systemProxyEnabled = computed(() => appStore.proxyMode === 'system')
const tunProxyEnabled = computed(() => appStore.proxyMode === 'tun')

// 代理模式配置 - 恢复原始实现
const proxyModes = [
  {
    value: 'system',
    nameKey: 'home.proxyMode.system',
    tipKey: 'home.proxyMode.systemTip',
    icon: GlobeOutline,
  },
  {
    value: 'tun',
    nameKey: 'home.proxyMode.tun',
    tipKey: 'home.proxyMode.tunTip',
    icon: FlashOutline,
  },
  {
    value: 'manual',
    nameKey: 'home.proxyMode.manual',
    tipKey: 'home.proxyMode.manualTip',
    icon: SettingsOutline,
  },
]

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

// 获取状态标题
const getStatusTitle = () => {
  if (isStarting.value) return t('status.starting')
  if (isStopping.value) return t('status.stopping')
  if (appStore.isRunning) {
    if (kernelStore.status.websocket_ready) return t('status.running')
    return t('status.disconnected')
  }
  return t('status.stopped')
}

// 获取状态描述
const getStatusSubtitle = () => {
  if (isStarting.value) return t('home.status.startingDesc')
  if (isStopping.value) return t('home.status.stoppingDesc')
  if (appStore.isRunning) {
    if (kernelStore.status.websocket_ready) return t('home.status.runningDesc')
    return t('home.status.disconnectedDesc')
  }
  return t('home.status.stoppedDesc')
}

// 获取当前代理模式名称
const getCurrentProxyModeName = () => {
  const mode = proxyModes.find((m) => m.value === appStore.proxyMode)
  return mode ? t(mode.nameKey) : t('common.unknown')
}

// 获取当前节点代理模式名称
const getCurrentNodeProxyModeName = () => {
  const mode = nodeProxyModes.find((m) => m.value === currentNodeProxyMode.value)
  return mode ? t(mode.nameKey) : t('common.unknown')
}

// 获取代理模式标签类型
const getProxyModeTagType = (mode: string) => {
  switch (mode) {
    case 'system':
      return 'info'
    case 'tun':
      return 'warning'
    case 'manual':
      return 'default'
    default:
      return 'default'
  }
}

// 获取节点代理模式标签类型
const getNodeProxyModeTagType = (mode: string) => {
  switch (mode) {
    case 'global':
      return 'info'
    case 'rule':
      return 'success'
    default:
      return 'default'
  }
}

const toggleSystemProxy = async (value: boolean) => {
  const targetMode: ProxyMode = value ? 'system' : tunProxyEnabled.value ? 'tun' : 'manual'
  await switchProxyModeAndRefreshKernel(targetMode)
}

const confirmTunSwitch = () => {
  return new Promise<boolean>((resolve) => {
    dialog.warning({
      title: t('home.tunConfirm.title'),
      content: t('home.tunConfirm.description'),
      positiveText: t('home.tunConfirm.confirm'),
      negativeText: t('common.cancel'),
      maskClosable: false,
      onPositiveClick: async () => {
        if (!isAdmin.value) {
          modeSwitchPending.value = true
          try {
            const success = await prepareTunModeWithAdminRestart()
            resolve(success)
          } finally {
            modeSwitchPending.value = false
          }
        } else {
          const success = await switchProxyModeAndRefreshKernel('tun')
          resolve(success)
        }
      },
      onNegativeClick: () => resolve(false),
    })
  })
}

const toggleTunProxy = async (value: boolean) => {
  if (value) {
    await confirmTunSwitch()
  } else {
    const fallbackMode: ProxyMode = systemProxyEnabled.value ? 'system' : 'manual'
    await switchProxyModeAndRefreshKernel(fallbackMode)
  }
}

// 重启内核
const restartKernel = async () => {
  if (isStarting.value || isStopping.value) return

  isStarting.value = true
  try {
    console.log('🔄 开始重启内核...')
    const result = await kernelStore.restartKernel({
      keepAlive: appStore.autoStartKernel,
    })
    if (result) {
      message.success(t('home.restartSuccess'))
      console.log('✅ 内核重启成功，数据收集已自动重启')
    } else {
      message.error(kernelStore.lastError || t('home.restartFailed'))
    }
  } catch (error) {
    console.error('重启内核失败:', error)
    message.error(t('home.restartFailed'))
  } finally {
    isStarting.value = false
  }
}

// 以管理员身份重启
const restartAsAdmin = async () => {
  try {
    const { systemApi } = await import('@/services/tauri')
    await systemApi.restartAsAdmin()
  } catch (error) {
    console.error('以管理员身份重启失败:', error)
    message.error(t('home.restartFailed'))
  }
}

// 切换代理模式并在必要时重启内核
const switchProxyModeAndRefreshKernel = async (mode: ProxyMode) => {
  const previousMode = appStore.proxyMode as ProxyMode
  if (modeSwitchPending.value || previousMode === mode) {
    return previousMode === mode
  }

  modeSwitchPending.value = true

  try {
    await appStore.setProxyMode(mode)
    const success = await kernelStore.switchProxyMode(mode)

    if (!success) {
      await appStore.setProxyMode(previousMode)
      return false
    }

    message.success(t('notification.proxyModeChanged'))
    return true
  } catch (error) {
    console.error('切换代理模式失败:', error)
    await appStore.setProxyMode(previousMode)
    message.error(t('notification.proxyModeChangeFailed'))
    return false
  } finally {
    modeSwitchPending.value = false
  }
}

const prepareTunModeWithAdminRestart = async () => {
  const previousMode = appStore.proxyMode as ProxyMode

  try {
    await appStore.setProxyMode('tun')
    await appStore.saveToBackend()
    await kernelStore.syncConfig()

    if (appStore.isRunning) {
      await kernelStore.stopKernel()
    }

    await restartAsAdmin()
    return true
  } catch (error) {
    console.error('保存 TUN 模式配置失败:', error)
    await appStore.setProxyMode(previousMode)
    message.error(t('home.restartFailed'))
    return false
  }
}

// 切换节点代理模式
const handleNodeProxyModeChange = async (mode: string) => {
  if (currentNodeProxyMode.value === mode) return

  try {
    const result = await kernelApi.switchNodeProxyMode(mode as 'global' | 'rule')
    console.log('节点代理模式切换结果:', result)
    currentNodeProxyMode.value = mode
    message.success(t('home.nodeModeChangeSuccess'))
  } catch (error) {
    console.error('切换节点代理模式失败:', error)
    message.error(t('home.nodeModeChangeFailed'))
  }
}

// 检查管理员权限
const checkAdmin = async () => {
  try {
    const { systemApi } = await import('@/services/tauri')
    isAdmin.value = await systemApi.checkAdmin()
  } catch (error) {
    console.error('检查管理员权限失败:', error)
    isAdmin.value = false
  }
}

// 生命周期
onMounted(async () => {
  checkAdmin()
  // 初始化KernelStore，确保数据收集正确设置
  await kernelStore.initializeStore()
})

onUnmounted(() => {
  // 清理工作
  console.log('🧹 HomeView卸载，清理数据监听')
  // 如果内核正在运行，保持数据收集继续
  // 如果需要完全清理，可以调用 stopDataCollection()
})
</script>

<style scoped>
.home-shell {
  padding: 24px;
  min-height: calc(100vh - 48px);
  background: v-bind('themeStore.isDark ? "#0f172a" : "#f6f7fb"');
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.hero-panel {
  border-radius: 28px;
  padding: 32px;
  background: v-bind(
    'themeStore.isDark ? "linear-gradient(135deg, #1d1f3a, #0f172a)" : "linear-gradient(135deg, #eef2ff, #fdf2f8)"'
  );
  border: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(15, 23, 42, 0.08)"');
  box-shadow: 0 25px 60px
    v-bind('themeStore.isDark ? "rgba(2, 6, 23, 0.8)" : "rgba(15, 23, 42, 0.08)"');
  overflow: hidden;
  position: relative;
}

.hero-main {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 32px;
  flex-wrap: wrap;
}

.hero-status {
  display: flex;
  align-items: center;
  gap: 20px;
  flex: 1;
  min-width: 240px;
}

.status-glow {
  width: 84px;
  height: 84px;
  border-radius: 24px;
  padding: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.4s ease;
  border: 1px solid rgba(255, 255, 255, 0.15);
}

.status-inner {
  width: 100%;
  height: 100%;
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(15, 23, 42, 0.35);
  box-shadow: inset 0 1px 8px rgba(15, 23, 42, 0.25);
}

.status-glow.orb-stopped {
  background: v-bind(
    'themeStore.isDark ? "linear-gradient(145deg, rgba(51, 65, 85, 0.7), rgba(30, 41, 59, 0.8))" : "linear-gradient(145deg, #f1f5f9, #e2e8f0)"'
  );
}

.status-glow.orb-connecting {
  background: v-bind(
    'themeStore.isDark ? "linear-gradient(145deg, rgba(245, 158, 11, 0.25), rgba(251, 191, 36, 0.25))" : "linear-gradient(145deg, rgba(245, 158, 11, 0.2), rgba(251, 191, 36, 0.2))"'
  );
  animation: hero-pulse 1.6s linear infinite;
}

.status-glow.orb-connected {
  background: v-bind(
    'themeStore.isDark ? "linear-gradient(145deg, rgba(16, 185, 129, 0.3), rgba(5, 150, 105, 0.4))" : "linear-gradient(145deg, rgba(16, 185, 129, 0.18), rgba(5, 150, 105, 0.25))"'
  );
}

.icon-stopped {
  color: #cbd5f5;
}

.icon-connecting {
  color: #fbbf24;
}

.icon-connected {
  color: #34d399;
}

.hero-texts {
  flex: 1;
}

.hero-title {
  font-size: 32px;
  margin: 0;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#0f172a"');
}

.hero-description {
  font-size: 15px;
  margin: 6px 0 16px;
  color: v-bind('themeStore.isDark ? "#cbd5f5" : "#475569"');
}

.hero-tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.hero-actions {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.rounded-control {
  border-radius: 16px;
  font-weight: 600;
  min-width: 130px;
  height: 42px;
  box-shadow: 0 10px 25px rgba(15, 23, 42, 0.12);
}

.hero-secondary {
  display: flex;
  gap: 12px;
}

.content-grid {
  margin-bottom: 32px;
}

.panel-card {
  background: v-bind('themeStore.isDark ? "rgba(15, 23, 42, 0.8)" : "rgba(255, 255, 255, 0.9)"');
  border-radius: 24px;
  border: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(15, 23, 42, 0.08)"');
  box-shadow: 0 20px 45px
    v-bind('themeStore.isDark ? "rgba(2, 6, 23, 0.7)" : "rgba(148, 163, 184, 0.25)"');
  overflow: hidden;
}

.panel-card :deep(.n-card__content) {
  padding: 24px;
}

.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.panel-eyebrow {
  margin: 0;
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#94a3b8"');
}

.panel-title {
  margin: 4px 0 0;
  font-size: 22px;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#0f172a"');
}

.mode-toggle-list {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.mode-toggle {
  border-radius: 20px;
  padding: 18px;
  border: 1px solid
    v-bind('themeStore.isDark ? "rgba(148, 163, 184, 0.2)" : "rgba(148, 163, 184, 0.4)"');
  background: v-bind(
    'themeStore.isDark ? "rgba(148, 163, 184, 0.05)" : "rgba(248, 250, 252, 0.9)"'
  );
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.mode-toggle-info {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
  min-width: 0;
}

.mode-grid {
  margin-top: 16px;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
}

.mode-card {
  border-radius: 20px;
  padding: 18px;
  border: 1px solid
    v-bind('themeStore.isDark ? "rgba(148, 163, 184, 0.2)" : "rgba(148, 163, 184, 0.4)"');
  background: v-bind(
    'themeStore.isDark ? "rgba(148, 163, 184, 0.08)" : "rgba(241, 245, 249, 0.7)"'
  );
  display: flex;
  align-items: center;
  gap: 16px;
  cursor: pointer;
  position: relative;
  transition: all 0.3s ease;
}

.mode-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 30px
    v-bind('themeStore.isDark ? "rgba(15, 23, 42, 0.7)" : "rgba(148, 163, 184, 0.35)"');
}

.mode-card.mode-active {
  border-color: #5b4cfd;
  background: v-bind(
    'themeStore.isDark ? "linear-gradient(135deg, rgba(91, 76, 253, 0.2), rgba(147, 51, 234, 0.25))" : "linear-gradient(135deg, rgba(91, 76, 253, 0.15), rgba(147, 51, 234, 0.2))"'
  );
}

.mode-icon {
  width: 48px;
  height: 48px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: v-bind('themeStore.isDark ? "rgba(15, 23, 42, 0.5)" : "rgba(255, 255, 255, 0.8)"');
  color: #5b4cfd;
}

.mode-name {
  margin: 0 0 6px;
  font-size: 16px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#0f172a"');
}

.mode-description {
  margin: 0;
  font-size: 13px;
  color: v-bind('themeStore.isDark ? "#cbd5f5" : "#475569"');
  line-height: 1.5;
}

.mode-indicator {
  margin-left: auto;
  display: flex;
  align-items: center;
}

.mode-indicator span {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: transparent;
  border: 2px solid transparent;
  transition: all 0.3s ease;
}

.mode-card.mode-active .mode-indicator span {
  border-color: #5b4cfd;
  box-shadow: 0 0 12px rgba(91, 76, 253, 0.6);
}

.traffic-card :deep(.n-card__content) {
  padding: 0;
}

@keyframes hero-pulse {
  0% {
    opacity: 0.8;
  }
  50% {
    opacity: 0.3;
  }
  100% {
    opacity: 0.8;
  }
}

@media (max-width: 1024px) {
  .hero-main {
    flex-direction: column;
  }

  .hero-actions {
    justify-content: flex-start;
  }
}

@media (max-width: 640px) {
  .home-shell {
    padding: 16px;
  }

  .hero-panel {
    padding: 24px;
  }

  .rounded-control {
    width: 100%;
  }

  .hero-actions {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
