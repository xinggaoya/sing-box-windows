<template>
  <div class="page-shell proxy-page" :style="pageThemeStyle">
    <section class="page-hero">
      <div class="hero-row">
        <div class="hero-left">
          <div class="hero-icon">
            <n-icon size="28">
              <SwapHorizontalOutline />
            </n-icon>
          </div>
          <div class="hero-meta">
            <p class="hero-subtitle">{{ t('proxy.subtitle') }}</p>
            <h2 class="hero-title">{{ t('proxy.title') }}</h2>
          </div>
        </div>
        <div class="hero-actions">
          <n-button
            @click="init"
            :loading="isLoading"
            type="primary"
            size="large"
          >
            <template #icon>
              <n-icon size="18">
                <RefreshOutline />
              </n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
        </div>
      </div>
      <div class="hero-stats">
        <div
          v-for="stat in proxyStats"
          :key="stat.label"
          class="stat-card"
          :data-accent="stat.accent"
        >
          <div class="stat-icon">
            <n-icon :size="20">
              <component :is="stat.icon" />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stat.value }}</div>
            <div class="stat-label">{{ stat.label }}</div>
          </div>
        </div>
      </div>
    </section>

    <section class="page-section">
      <n-spin :show="isLoading" class="loading-container">
        <template #description>
          <span class="loading-text">{{ t('proxy.loadingInfo') }}</span>
        </template>

        <!-- 空状态 -->
        <div v-if="proxyGroups.length === 0 && !isLoading" class="empty-state">
          <div class="empty-icon">
            <n-icon size="48"><GlobeOutline /></n-icon>
          </div>
          <h3 class="empty-title">{{ t('proxy.noProxyGroups') }}</h3>
          <p class="empty-description">{{ t('proxy.checkConfigOrRefresh') }}</p>
          <n-button @click="init" type="primary" size="medium">
            <template #icon>
              <n-icon size="16"><RefreshOutline /></n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
        </div>

        <!-- 代理组列表 -->
        <div v-else class="proxy-groups">
          <div
            v-for="group in [...proxyGroups].reverse()"
            :key="group.name"
            class="proxy-group"
          >
            <!-- 组头部 -->
            <div class="group-header" @click="toggleGroup(group.name)">
              <div class="group-info">
                <div class="group-main">
                  <h3 class="group-name">{{ group.name }}</h3>
                  <div class="group-badges">
                    <n-tag size="small" type="info" round :bordered="false">
                      {{ group.all.length }} {{ t('proxy.nodes') }}
                    </n-tag>
                    <n-tag size="small" type="default" round :bordered="false">
                      {{ group.type }}
                    </n-tag>
                  </div>
                </div>
                <div class="group-current">
                  <span class="current-label">{{ t('proxy.currentLabel') }}:</span>
                  <n-tag type="success" size="small" round :bordered="false">
                    {{ group.now }}
                  </n-tag>
                </div>
              </div>

              <div class="group-actions">
                <n-button
                  @click.stop="testNodeDelay(group.name)"
                  :loading="testingGroup === group.name"
                  type="info"
                  size="small"
                  class="test-btn"
                >
                  <template #icon>
                    <n-icon size="14"><SpeedometerOutline /></n-icon>
                  </template>
                  {{ t('proxy.testNode') }}
                </n-button>
                <div class="expand-icon" :class="{ expanded: expandedGroups.includes(group.name) }">
                  <n-icon size="18">
                    <ChevronDownOutline />
                  </n-icon>
                </div>
              </div>
            </div>

            <!-- 节点网格 -->
            <transition name="group-expand">
              <div v-if="expandedGroups.includes(group.name)" class="nodes-container">
                <div class="nodes-grid" :style="{ '--grid-columns': gridCols }">
                  <div
                    v-for="(proxy, i) in group.all"
                    :key="`${group.name}-${proxy}-${i}`"
                    class="node-card"
                    :class="{
                      'node-active': group.now === proxy,
                      'node-testing': testingNodes[proxy],
                    }"
                    @click="changeProxy(group.name, proxy)"
                  >
                    <!-- 节点状态指示器 -->
                    <div class="node-status" :class="getNodeStatusType(proxy)">
                      <n-icon v-if="group.now === proxy" size="16">
                        <CheckmarkCircleOutline />
                      </n-icon>
                      <n-icon v-else-if="testingNodes[proxy]" size="16" class="spin">
                        <RefreshOutline />
                      </n-icon>
                      <div v-else class="status-dot"></div>
                    </div>

                    <!-- 节点信息 -->
                    <div class="node-info">
                      <div class="node-name" :title="proxy">{{ proxy }}</div>
                      <div class="node-delay" @click.stop="testSingleNode(proxy)">
                        <span class="delay-value">{{ getNodeStatusText(proxy) }}</span>
                      </div>
                    </div>

                    <!-- 活跃指示线 -->
                    <div v-if="group.now === proxy" class="active-indicator"></div>
                  </div>
                </div>
              </div>
            </transition>
          </div>
        </div>
      </n-spin>
    </section>
  </div>
</template>

<script lang="ts" setup>
import { onMounted, ref, computed, reactive, onUnmounted } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  CheckmarkCircleOutline,
  SwapHorizontalOutline,
  SpeedometerOutline,
  GlobeOutline,
  ChevronDownOutline,
} from '@vicons/ionicons5'
import { useWindowSize } from '@vueuse/core'
import { tauriApi } from '@/services/tauri'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { usePageTheme } from '@/composables/usePageTheme'

// 接口定义
interface ProxyHistory {
  time: string
  delay: number
}

interface ProxyData {
  type: string
  name: string
  now: string
  all: string[]
  history: ProxyHistory[]
  udp: boolean
}

interface Proxies {
  proxies: Record<string, ProxyData>
}

interface TestGroupResult {
  [proxyName: string]: number
}

interface TestNodeResult {
  proxy: string
  delay: number
}

// 状态定义
defineOptions({
  name: 'ProxyView'
})

const message = useMessage()
const isLoading = ref(false)
const { width } = useWindowSize()
const { t } = useI18n()
const appStore = useAppStore()
const themeStore = useThemeStore()
const pageThemeStyle = usePageTheme(themeStore)

// 代理数据
const rawProxies = ref<Record<string, ProxyData>>({})
const proxyGroups = ref<ProxyData[]>([])
const testingNodes = reactive<Record<string, boolean>>({})
const expandedGroups = ref<string[]>([])

// 注册事件监听器
let unlistenTestProgress: (() => void) | null = null
let unlistenTestResult: (() => void) | null = null
let unlistenTestComplete: (() => void) | null = null
let unlistenNodeResult: (() => void) | null = null

// 测试状态和结果
const testingGroup = ref('')
const testResults = reactive<Record<string, number>>({})
const nodeErrors = reactive<Record<string, string>>({})

// 切换组展开状态
const toggleGroup = (groupName: string) => {
  const index = expandedGroups.value.indexOf(groupName)
  if (index > -1) {
    expandedGroups.value.splice(index, 1)
  } else {
    expandedGroups.value.push(groupName)
  }
}

// 根据窗口宽度调整网格列数
const gridCols = computed(() => {
  if (width.value < 640) return 1
  if (width.value < 960) return 2
  if (width.value < 1280) return 3
  if (width.value < 1600) return 4
  return 5
})

const proxyStats = computed(() => {
  const groups = proxyGroups.value
  const totalNodes = groups.reduce((sum, group) => sum + (group.all?.length ?? 0), 0)
  const expanded = expandedGroups.value.length
  const testingCount = Object.values(testingNodes).filter(Boolean).length

  return [
    {
      label: t('proxy.dashboard.groupTotal'),
      value: groups.length,
      icon: SwapHorizontalOutline,
      accent: 'purple',
    },
    {
      label: t('proxy.dashboard.nodeTotal'),
      value: totalNodes,
      icon: GlobeOutline,
      accent: 'blue',
    },
    {
      label: t('proxy.dashboard.expanded'),
      value: expanded,
      icon: ChevronDownOutline,
      accent: 'amber',
    },
    {
      label: t('proxy.dashboard.testing'),
      value: testingCount,
      icon: SpeedometerOutline,
      accent: 'pink',
    },
  ]
})

// 生命周期钩子
onMounted(() => {
  init()
  setupEventListeners()
})

onUnmounted(() => {
  if (unlistenTestProgress) unlistenTestProgress()
  if (unlistenTestResult) unlistenTestResult()
  if (unlistenTestComplete) unlistenTestComplete()
  if (unlistenNodeResult) unlistenNodeResult()
})

// 设置事件监听器
const setupEventListeners = async () => {
  unlistenTestProgress = await listen('test-nodes-progress', (event) => {
    const data = event.payload as { current: number; total: number; node: string; status: string }
    console.log(t('proxy.testProgress'), data)
  })

  unlistenTestResult = await listen('proxy-group-delay-result', (event) => {
    const data = event.payload as TestGroupResult
    console.log('收到组延迟测试结果:', data)

    if (data && typeof data === 'object') {
      Object.entries(data).forEach(([proxyName, delay]) => {
        if (typeof delay === 'number') {
          testResults[proxyName] = delay
        }
      })
      message.success(t('proxy.groupTestComplete'))
    } else {
      message.error(`${t('proxy.testFailed')}: ${JSON.stringify(data)}`)
    }
    testingGroup.value = ''
  })

  unlistenTestComplete = await listen('test-nodes-complete', () => {
    message.success(t('proxy.batchTestComplete'))
  })

  unlistenNodeResult = await listen('proxy-delay-result', (event) => {
    const data = event.payload as TestNodeResult
    console.log('收到节点延迟测试结果:', data)

    if (data && data.proxy) {
      const { proxy, delay } = data

      testingNodes[proxy] = false

      if (delay !== undefined && delay > 0) {
        testResults[proxy] = delay
        delete nodeErrors[proxy]
        message.success(`${t('proxy.nodeTestComplete')}: ${proxy} (${delay}ms)`)
      } else {
        nodeErrors[proxy] = t('proxy.timeout')
        message.warning(`${proxy}: ${t('proxy.timeout')}`)
      }
    } else {
      message.error(`${t('proxy.nodeTestFailed')}: ${JSON.stringify(data)}`)
    }
  })
}

/**
 * 初始化并获取代理信息
 */
const init = async () => {
  isLoading.value = true
  try {
    const data = await tauriApi.proxy.getProxies()
    rawProxies.value = data.proxies

    // 提取代理组
    const groups: ProxyData[] = []

    Object.entries(data.proxies).forEach(([key, item]) => {
      // 排除特殊组和直连
      if (key === 'GLOBAL' || key === 'direct') return

      // 只显示Selector类型的代理组
      if (item.type === 'Selector') {
        groups.push(item)
      }
    })

    proxyGroups.value = groups

    if (groups.length > 0) {
      message.success(t('proxy.loadSuccess'))
    }
  } catch (error) {
    console.error(t('proxy.loadFailed'), error)
    message.error(t('proxy.loadFailedCheck'))
  } finally {
    isLoading.value = false
  }
}

/**
 * 获取节点状态对应的颜色类型
 */
const getNodeStatusType = (name: string): string => {
  if (nodeErrors[name]) return 'error'
  if (testingNodes[name]) return 'info'

  const delay = testResults[name] || 0
  if (delay === 0) return 'default'
  if (delay < 100) return 'success'
  if (delay < 200) return 'info'
  if (delay < 300) return 'warning'
  return 'error'
}

/**
 * 获取节点状态文本
 */
const getNodeStatusText = (name: string): string => {
  if (testingNodes[name]) return t('proxy.testing')
  if (nodeErrors[name]) return t('proxy.timeout')

  const delay = testResults[name] || 0
  if (delay === 0) return '--'
  return `${delay}ms`
}

/**
 * 测试单个节点延迟
 */
const testSingleNode = async (proxy: string) => {
  if (testingNodes[proxy]) return

  testingNodes[proxy] = true

  try {
    delete nodeErrors[proxy]
    console.log(`开始测试节点延迟: ${proxy}, API端口: ${appStore.apiPort}`)
    await tauriApi.proxy.testNodeDelay(proxy)
  } catch (error) {
    console.error(`测试节点 ${proxy} 失败:`, error)
    message.error(`${t('proxy.testErrorMessage')}: ${proxy}`)
    testingNodes[proxy] = false
    nodeErrors[proxy] = String(error)
    testingNodes[proxy] = false
    nodeErrors[proxy] = t('proxy.timeout')
  }
}

/**
 * 测试节点延迟
 */
const testNodeDelay = async (group: string) => {
  if (testingGroup.value === group) return

  testingGroup.value = group
  try {
    console.log(`开始测试组延迟: ${group}, API端口: ${appStore.apiPort}`)
    await tauriApi.proxy.testGroupDelay(group)
    console.log(`组延迟测试请求已发送: ${group}`)
  } catch (error) {
    console.error(`测试组 ${group} 失败:`, error)
    message.error(`${t('proxy.testErrorMessage')}: ${group}`)
    testingGroup.value = ''
  }
}

/**
 * 切换代理
 */
const changeProxy = async (group: string, proxy: string) => {
  try {
    await tauriApi.proxy.changeProxy(group, proxy)
    message.success(t('proxy.switchSuccess', { group: group, proxy: proxy }))
    // 重新加载数据
    await init()
    // 重新测试当前组
    await testNodeDelay(group)
  } catch (error) {
    console.error(t('proxy.switchFailed'), error)
    message.error(t('proxy.switchErrorMessage'))
  }
}
</script>

<style scoped>
.proxy-page {
  animation: fadeIn 0.4s ease both;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.loading-container {
  min-height: 320px;
}

.loading-text {
  color: var(--text-muted);
  font-size: 14px;
}

.proxy-groups {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.proxy-group {
  border-radius: 28px;
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  box-shadow: var(--panel-shadow);
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 22px 28px;
  cursor: pointer;
  border-bottom: 1px solid var(--divider-color);
  transition: background 0.2s ease;
}

.group-header:hover {
  background: rgba(91, 76, 253, 0.08);
}

.group-info {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
  flex: 1;
}

.group-main {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
}

.group-name {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
}

.group-badges {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.group-current {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: var(--text-muted);
}

.group-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.expand-icon {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  background: var(--chip-bg);
  color: var(--text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.25s ease;
}

.expand-icon.expanded {
  transform: rotate(180deg);
}

.nodes-container {
  padding: 24px 28px 32px;
}

.nodes-grid {
  display: grid;
  grid-template-columns: repeat(var(--grid-columns, 4), minmax(0, 1fr));
  gap: 14px;
}

.node-card {
  position: relative;
  border-radius: 20px;
  padding: 18px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--panel-border);
  display: flex;
  gap: 14px;
  align-items: center;
  cursor: pointer;
  transition: transform 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}

.node-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 25px 40px rgba(15, 23, 42, 0.12);
}

.node-card.node-active {
  border-color: #10b981;
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.18), rgba(5, 150, 105, 0.22));
}

.node-card.node-testing {
  border-color: #6366f1;
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.16), rgba(59, 130, 246, 0.18));
}

.node-status {
  width: 36px;
  height: 36px;
  border-radius: 14px;
  background: var(--chip-bg);
  color: var(--text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.node-status.success {
  background: rgba(16, 185, 129, 0.18);
  color: #059669;
}

.node-status.info {
  background: rgba(59, 130, 246, 0.18);
  color: #2563eb;
}

.node-status.warning {
  background: rgba(251, 191, 36, 0.2);
  color: #b45309;
}

.node-status.error {
  background: rgba(248, 113, 113, 0.2);
  color: #dc2626;
}

.node-info {
  flex: 1;
  min-width: 0;
}

.node-name {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-delay {
  margin-top: 4px;
  font-size: 13px;
  color: var(--text-muted);
}

.active-indicator {
  position: absolute;
  inset: 0;
  border-radius: 20px;
  border: 1px solid rgba(16, 185, 129, 0.6);
  pointer-events: none;
}

.group-expand-enter-active,
.group-expand-leave-active {
  overflow: hidden;
  transition: all 0.3s ease;
}

.group-expand-enter-from,
.group-expand-leave-to {
  opacity: 0;
  transform: translateY(-12px);
}

@media (max-width: 1024px) {
  .group-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .group-actions {
    width: 100%;
    justify-content: space-between;
  }
}

@media (max-width: 768px) {
  .nodes-grid {
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  }
}

@media (max-width: 560px) {
  .nodes-container {
    padding: 18px;
  }

  .group-actions {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
