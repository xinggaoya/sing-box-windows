<template>
  <div class="modern-home">
    <!-- 状态卡片区域 -->
    <div class="status-section">
      <n-card class="status-card" :bordered="false">
        <div class="status-content">
          <!-- 左侧状态信息 -->
          <div class="status-left">
            <div class="status-visual">
              <div class="status-orb" :class="statusOrbClass">
                <div class="orb-core"></div>
                <div class="orb-pulse" v-if="appStore.isRunning"></div>
                <div class="orb-icon">
                  <n-icon :size="28" :class="statusIconClass">
                    <PowerOutline v-if="!appStore.isRunning" />
                    <CheckmarkCircleOutline v-else-if="kernelStore.status.websocket_ready" />
                    <TimeOutline v-else-if="isStarting" />
                    <CloseCircleOutline v-else />
                  </n-icon>
                </div>
              </div>
            </div>

            <div class="status-info">
              <h2 class="status-title">{{ getStatusTitle() }}</h2>
              <p class="status-description">{{ getStatusSubtitle() }}</p>

              <!-- 状态标签 -->
              <div class="status-badges">
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
                  {{ kernelStore.status.websocket_ready ? t('home.wsStatus.connected') : t('home.wsStatus.disconnected') }}
                </n-tag>

                <n-tag
                  :type="isAdmin ? 'success' : 'warning'"
                  size="small"
                  round
                  :bordered="false"
                >
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

          <!-- 右侧控制按钮 -->
          <div class="status-controls">
            <div class="main-control-btn">
              <n-button
                v-if="!appStore.isRunning"
                type="primary"
                size="large"
                :loading="isStarting"
                @click="runKernel"
                class="start-btn"
              >
                <template #icon>
                  <n-icon :size="20">
                    <PlayOutline />
                  </n-icon>
                </template>
                {{ t('home.start') }}
              </n-button>

              <n-button
                v-else
                type="error"
                size="large"
                :loading="isStopping"
                @click="stopKernel"
                class="stop-btn"
              >
                <template #icon>
                  <n-icon :size="20">
                    <StopCircleOutline />
                  </n-icon>
                </template>
                {{ t('home.stop') }}
              </n-button>
            </div>

            <div class="secondary-controls">
              <n-tooltip :show-arrow="false">
                <template #trigger>
                  <n-button
                    circle
                    quaternary
                    size="medium"
                    @click="restartKernel"
                    :loading="isStarting || isStopping"
                  >
                    <n-icon :size="18">
                      <RefreshOutline />
                    </n-icon>
                  </n-button>
                </template>
                {{ t('home.restart') }}
              </n-tooltip>

              <n-tooltip :show-arrow="false" v-if="!isAdmin">
                <template #trigger>
                  <n-button
                    circle
                    quaternary
                    size="medium"
                    @click="restartAsAdmin"
                  >
                    <n-icon :size="18">
                      <ShieldCheckmarkOutline />
                    </n-icon>
                  </n-button>
                </template>
                {{ t('home.restartAsAdmin') }}
              </n-tooltip>
            </div>
          </div>
        </div>
      </n-card>
    </div>

    <!-- 主要内容区域 -->
    <div class="main-content">
      <n-grid
        :cols="24"
        :x-gap="12"
        :y-gap="12"
        responsive="screen"
      >
        <!-- 左侧：代理模式配置 -->
        <n-grid-item :span="24" :s="24" :m="16" :l="16" :xl="16" :xxl="16">
          <!-- 流量代理模式 -->
          <n-card class="config-card" :bordered="false">
            <div class="card-header">
              <div class="header-left">
                <n-icon :size="20" class="header-icon">
                  <GlobeOutline />
                </n-icon>
                <h3 class="header-title">{{ t('home.proxyHeader.flowMode') }}</h3>
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

            <div class="proxy-modes">
              <div
                v-for="mode in proxyModes"
                :key="mode.value"
                class="mode-card"
                :class="{ 'mode-active': appStore.proxyMode === mode.value }"
                @click="onModeChange(mode.value)"
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
                  <div class="indicator-dot"></div>
                </div>
              </div>
            </div>
          </n-card>

          <!-- 节点代理模式 -->
          <n-card class="config-card" :bordered="false">
            <div class="card-header">
              <div class="header-left">
                <n-icon :size="20" class="header-icon">
                  <SettingsOutline />
                </n-icon>
                <h3 class="header-title">{{ t('home.proxyHeader.nodeMode') }}</h3>
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

            <div class="node-proxy-modes">
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
                  <div class="indicator-dot"></div>
                </div>
              </div>
            </div>
          </n-card>
        </n-grid-item>

        <!-- 右侧：流量统计和系统信息 -->
        <n-grid-item :span="24" :s="24" :m="8" :l="8" :xl="8" :xxl="8">
          <!-- 流量统计卡片（包含图表） -->
          <TrafficStatsCard />
        </n-grid-item>
      </n-grid>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  NCard,
  NButton,
  NIcon,
  NGrid,
  NGridItem,
  NTag,
  NSpin,
  NTooltip,
  useMessage
} from 'naive-ui'
import {
  PlayOutline,
  StopCircleOutline,
  RefreshOutline,
  SettingsOutline,
  GlobeOutline,
  FlashOutline,
  ShieldCheckmarkOutline,
  SpeedometerOutline,
  PowerOutline,
  CheckmarkCircleOutline,
  TimeOutline,
  CloseCircleOutline,
  WifiOutline,
  ArrowUpOutline,
  ArrowDownOutline,
  SwapHorizontalOutline,
  ExtensionPuzzleOutline,
  RadioOutline
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { kernelApi } from '@/services/tauri-api'
import { formatBandwidth } from '@/utils'
import TrafficStatsCard from '@/components/home/TrafficStatsCard.vue'

defineOptions({
  name: 'HomeView'
})

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const message = useMessage()

// Store实例
const appStore = useAppStore()
const proxyStore = useProxyStore()
const kernelStore = useKernelStore()
const trafficStore = useTrafficStore()
const themeStore = useThemeStore()

// 响应式状态
const isStarting = ref(false)
const isStopping = ref(false)
const isAdmin = ref(false)
const currentNodeProxyMode = ref('rule')

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
  const mode = proxyModes.find(m => m.value === appStore.proxyMode)
  return mode ? t(mode.nameKey) : t('common.unknown')
}

// 获取当前节点代理模式名称
const getCurrentNodeProxyModeName = () => {
  const mode = nodeProxyModes.find(m => m.value === currentNodeProxyMode.value)
  return mode ? t(mode.nameKey) : t('common.unknown')
}

// 获取代理模式标签类型
const getProxyModeTagType = (mode: string) => {
  switch (mode) {
    case 'system': return 'info'
    case 'tun': return 'warning'
    case 'manual': return 'default'
    default: return 'default'
  }
}

// 获取节点代理模式标签类型
const getNodeProxyModeTagType = (mode: string) => {
  switch (mode) {
    case 'global': return 'info'
    case 'rule': return 'success'
    default: return 'default'
  }
}

// 启动内核
const runKernel = async () => {
  if (isStarting.value || isStopping.value) return

  isStarting.value = true
  try {
    const result = await kernelApi.startKernel()
    if (result.success) {
      message.success(t('home.startSuccess'))

      // 启动成功后等待片刻再刷新状态
      setTimeout(async () => {
        await refreshKernelStatus()
      }, 1000)
    } else {
      message.error(result.message || t('home.startFailed'))
    }
  } catch (error) {
    console.error('启动内核失败:', error)
    message.error(t('home.startFailed'))
  } finally {
    isStarting.value = false
  }
}

// 停止内核
const stopKernel = async () => {
  if (isStarting.value || isStopping.value) return

  isStopping.value = true
  try {
    const result = await kernelApi.stopKernel()
    if (result.success) {
      message.success(t('home.stopSuccess'))

      // 停止成功后立即刷新状态
      await refreshKernelStatus()
    } else {
      message.error(result.message || t('home.stopFailed'))
    }
  } catch (error) {
    console.error('停止内核失败:', error)
    // 检查错误消息，如果包含"成功"字样，则显示成功
    const errorMsg = error instanceof Error ? error.message : String(error)
    if (errorMsg.includes('成功') || errorMsg.includes('success')) {
      message.success(t('home.stopSuccess'))
      // 即使出错也尝试刷新状态
      await refreshKernelStatus()
    } else {
      message.error(t('home.stopFailed'))
    }
  } finally {
    isStopping.value = false
  }
}

// 重启内核
const restartKernel = async () => {
  if (isStarting.value || isStopping.value) return

  isStarting.value = true
  try {
    const result = await kernelApi.restartKernel()
    if (result.success) {
      message.success(t('home.startSuccess'))
    } else {
      message.error(result.message || t('home.restartFailed'))
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
    const { systemApi } = await import('@/services/tauri-api')
    await systemApi.restartAsAdmin()
  } catch (error) {
    console.error('以管理员身份重启失败:', error)
    message.error(t('home.restartFailed'))
  }
}

// 切换代理模式
const onModeChange = async (mode: string) => {
  if (appStore.proxyMode === mode || isStarting.value || isStopping.value) return

  try {
    const result = await kernelApi.switchProxyMode(mode as 'system' | 'tun' | 'manual')
    console.log('代理模式切换结果:', result)
    appStore.setProxyMode(mode as any)
    message.success(t('notification.proxyModeChanged'))
  } catch (error) {
    console.error('切换代理模式失败:', error)
    message.error(t('notification.proxyModeChangeFailed'))
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

// 刷新内核状态
const refreshKernelStatus = async () => {
  try {
    console.log('🔄 刷新内核状态...')

    // 强制刷新状态
    const { kernelService } = await import('@/services/kernel-service')
    const newStatus = await kernelService.forceRefreshStatus()

    console.log('📊 内核状态已更新:', newStatus)

    // 更新 store 状态
    await kernelStore.syncStatus()
  } catch (error) {
    console.error('刷新内核状态失败:', error)
  }
}

// 检查管理员权限
const checkAdmin = async () => {
  try {
    const { systemApi } = await import('@/services/tauri-api')
    isAdmin.value = await systemApi.checkAdmin()
  } catch (error) {
    console.error('检查管理员权限失败:', error)
    isAdmin.value = false
  }
}

// 生命周期
onMounted(() => {
  checkAdmin()
})

onUnmounted(() => {
  // 清理工作
})
</script>

<style scoped>
/* 现代化主页样式 */
.modern-home {
  padding: 16px;
  background: v-bind('themeStore.isDark ? "#18181b" : "#f8fafc"');
  min-height: calc(100vh - 48px);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
}

/* 状态卡片区域 */
.status-section {
  margin-bottom: 16px;
}

.status-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  border-radius: 20px;
  box-shadow: 0 8px 32px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.03)"');
  transition: all 0.3s ease;
}

.status-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 40px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.05)"');
}

.status-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px;
  gap: 16px;
}

/* 左侧状态信息 */
.status-left {
  display: flex;
  align-items: center;
  gap: 20px;
  flex: 1;
}

.status-visual {
  position: relative;
}

.status-orb {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: all 0.4s ease;
}

.orb-core {
  position: absolute;
  inset: 8px;
  border-radius: 50%;
  background: v-bind('themeStore.isDark ? "radial-gradient(circle, #374151, #1f2937)" : "radial-gradient(circle, #e5e7eb, #d1d5db)"');
  transition: all 0.3s ease;
}

.orb-pulse {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: v-bind('themeStore.isDark ? "rgba(91, 76, 253, 0.2)" : "rgba(91, 76, 253, 0.15)"');
  animation: orb-pulse 2s ease-in-out infinite;
}

.orb-icon {
  position: relative;
  z-index: 2;
  transition: all 0.3s ease;
}

/* 状态球颜色 */
.orb-stopped {
  background: v-bind('themeStore.isDark ? "conic-gradient(from 0deg, #6b7280, #9ca3af, #6b7280)" : "conic-gradient(from 0deg, #d1d5db, #e5e7eb, #d1d5db)"');
}

.orb-connecting {
  background: v-bind('themeStore.isDark ? "conic-gradient(from 0deg, #f59e0b, #fbbf24, #f59e0b)" : "conic-gradient(from 0deg, #f59e0b, #fbbf24, #f59e0b)"');
  animation: orb-rotate 1s linear infinite;
}

.orb-connected {
  background: v-bind('themeStore.isDark ? "conic-gradient(from 0deg, #10b981, #34d399, #10b981)" : "conic-gradient(from 0deg, #10b981, #34d399, #10b981)"');
}

@keyframes orb-pulse {
  0%, 100% { transform: scale(1); opacity: 0.3; }
  50% { transform: scale(1.2); opacity: 0.1; }
}

@keyframes orb-rotate {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.status-info {
  flex: 1;
}

.status-title {
  font-size: 28px;
  font-weight: 700;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0 0 8px 0;
  letter-spacing: -0.02em;
}

.status-description {
  font-size: 14px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0 0 16px 0;
  line-height: 1.5;
}

.status-badges {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

/* 右侧控制按钮 */
.status-controls {
  display: flex;
  align-items: center;
  gap: 16px;
}

.main-control-btn {
  position: relative;
}

.start-btn,
.stop-btn {
  width: 140px;
  height: 48px;
  border-radius: 16px;
  font-weight: 600;
  font-size: 15px;
  transition: all 0.3s ease;
  border: none;
  position: relative;
  overflow: hidden;
}

.start-btn {
  background: linear-gradient(135deg, #10b981, #059669);
  color: white;
}

.start-btn:hover {
  background: linear-gradient(135deg, #059669, #047857);
  transform: translateY(-1px);
  box-shadow: 0 8px 25px rgba(16, 185, 129, 0.3);
}

.stop-btn {
  background: linear-gradient(135deg, #ef4444, #dc2626);
  color: white;
}

.stop-btn:hover {
  background: linear-gradient(135deg, #dc2626, #b91c1c);
  transform: translateY(-1px);
  box-shadow: 0 8px 25px rgba(239, 68, 68, 0.3);
}

.secondary-controls {
  display: flex;
  gap: 8px;
}

/* 主要内容区域 */
.main-content {
  margin-top: 16px;
}

/* 配置卡片 */
.config-card,
.stats-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  border-radius: 20px;
  box-shadow: 0 8px 32px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.03)"');
  transition: all 0.3s ease;
  margin-bottom: 12px;
}

.config-card:hover,
.stats-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 40px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.05)"');
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding-bottom: 16px;
  border-bottom: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon {
  color: v-bind('themeStore.isDark ? "#5b4cfd" : "#5b4cfd"');
}

.header-title {
  font-size: 18px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0;
}

/* 代理模式卡片 */
.proxy-modes,
.node-proxy-modes {
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
  border: 2px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.02)" : "rgba(0, 0, 0, 0.02)"');
  cursor: pointer;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.mode-card:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.04)"');
  border-color: v-bind('themeStore.isDark ? "rgba(91, 76, 253, 0.3)" : "rgba(91, 76, 253, 0.2)"');
  transform: translateX(4px);
}

.mode-card.mode-active {
  background: v-bind('themeStore.isDark ? "rgba(91, 76, 253, 0.15)" : "rgba(91, 76, 253, 0.1)"');
  border-color: #5b4cfd;
}

.mode-card.mode-active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: linear-gradient(180deg, #5b4cfd, #7c3aed);
  border-radius: 0 4px 4px 0;
}

.mode-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  transition: all 0.3s ease;
}

.mode-card.mode-active .mode-icon {
  background: v-bind('themeStore.isDark ? "rgba(91, 76, 253, 0.2)" : "rgba(91, 76, 253, 0.15)"');
  color: #5b4cfd;
}

.mode-content {
  flex: 1;
}

.mode-name {
  font-size: 16px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0 0 4px 0;
}

.mode-description {
  font-size: 13px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0;
  line-height: 1.4;
}

.mode-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.mode-card.mode-active .mode-indicator {
  opacity: 1;
}

.indicator-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #5b4cfd;
  box-shadow: 0 0 12px rgba(91, 76, 253, 0.5);
  animation: indicator-pulse 2s ease-in-out infinite;
}

@keyframes indicator-pulse {
  0%, 100% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.2); opacity: 0.7; }
}

/* 流量统计 */
.traffic-stats {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 12px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.03)" : "rgba(0, 0, 0, 0.03)"');
  transition: all 0.2s ease;
}

.stat-item:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.06)"');
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 10px;
  color: white;
  font-weight: 600;
}

.stat-icon.upload {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.stat-icon.download {
  background: linear-gradient(135deg, #10b981, #059669);
}

.stat-icon.total {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.stat-icon.memory {
  background: linear-gradient(135deg, #f59e0b, #d97706);
}

.stat-content {
  flex: 1;
}

.stat-label {
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0 0 2px 0;
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .modern-home {
    padding: 16px;
  }

  .status-content {
    flex-direction: column;
    gap: 16px;
    text-align: center;
  }

  .status-badges {
    justify-content: center;
  }

  .status-controls {
    justify-content: center;
  }

  .mode-card {
    padding: 12px;
  }

  .mode-icon {
    width: 40px;
    height: 40px;
  }
}

@media (max-width: 480px) {
  .status-orb {
    width: 60px;
    height: 60px;
  }

  .status-title {
    font-size: 24px;
  }

  .start-btn,
  .stop-btn {
    width: 120px;
    height: 42px;
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .modern-home {
    padding: 16px;
  }

  .status-content {
    flex-direction: column;
    gap: 16px;
    text-align: center;
  }

  .status-controls {
    flex-direction: column;
    gap: 12px;
  }

  .main-control-btn,
  .secondary-controls {
    width: 100%;
    justify-content: center;
  }

  .proxy-modes,
  .node-proxy-modes {
    gap: 8px;
  }

  .mode-card {
    padding: 12px;
    gap: 12px;
  }

  .mode-icon {
    width: 40px;
    height: 40px;
  }

  .mode-name {
    font-size: 14px;
  }

  .mode-description {
    font-size: 12px;
  }

  .status-orb {
    width: 80px;
    height: 80px;
  }

  .status-title {
    font-size: 20px;
  }

  .status-description {
    font-size: 14px;
  }
}

@media (max-width: 480px) {
  .modern-home {
    padding: 12px;
  }

  .mode-card {
    flex-direction: column;
    text-align: center;
    gap: 8px;
    padding: 16px 12px;
  }

  .mode-content {
    width: 100%;
  }

  .mode-indicator {
    position: absolute;
    top: 8px;
    right: 8px;
  }

  .config-card {
    margin-bottom: 16px;
  }

  .card-header {
    flex-direction: column;
    gap: 8px;
    text-align: center;
  }

  .status-badges {
    justify-content: center;
    flex-wrap: wrap;
  }
}
</style>