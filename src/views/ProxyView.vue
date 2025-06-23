<template>
  <div class="modern-proxy">
    <!-- 顶部操作栏 -->
    <div class="proxy-header">
      <div class="header-info">
        <div class="page-icon">
          <n-icon size="24">
            <SwapHorizontalOutline />
          </n-icon>
        </div>
        <div class="page-title-section">
          <h1 class="page-title">{{ t('proxy.title') }}</h1>
          <p class="page-subtitle">{{ t('proxy.subtitle') }}</p>
        </div>
      </div>

      <div class="header-actions">
        <n-button
          @click="init"
          :loading="isLoading"
          type="primary"
          size="medium"
          round
          class="refresh-btn"
        >
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          {{ t('proxy.refreshList') }}
        </n-button>
      </div>
    </div>

    <!-- 代理组内容 -->
    <div class="proxy-main">
      <n-spin :show="isLoading" class="loading-container">
        <template #description>
          <span class="loading-text">{{ t('proxy.loadingInfo') }}</span>
        </template>

        <div v-if="proxyGroups.length === 0 && !isLoading" class="empty-state">
          <div class="empty-icon">
            <n-icon size="48">
              <GlobeOutline />
            </n-icon>
          </div>
          <div class="empty-title">{{ t('proxy.noProxyGroups') }}</div>
          <div class="empty-description">{{ t('proxy.checkConfigOrRefresh') }}</div>
        </div>

        <div v-else class="proxy-groups">
          <div v-for="group in [...proxyGroups].reverse()" :key="group.name" class="group-section">
            <!-- 组头部信息 -->
            <div class="group-header">
              <div class="group-title-section">
                <h3 class="group-title">{{ group.name }}</h3>
                <div class="group-meta">
                  <n-tag size="small" round type="info">
                    {{ group.all.length }} {{ t('proxy.nodeCount') }}
                  </n-tag>
                  <n-tag size="small" round type="default">
                    {{ group.type }}
                  </n-tag>
                </div>
              </div>

              <div class="group-actions">
                <div class="current-node-info">
                  <span class="current-label">{{ t('proxy.currentLabel') }}</span>
                  <n-tag type="success" size="small" round>
                    <template #icon>
                      <n-icon><CheckmarkCircleOutline /></n-icon>
                    </template>
                    {{ group.now }}
                  </n-tag>
                </div>

                <n-button
                  @click="testNodeDelay(group.name)"
                  :loading="testingGroup === group.name"
                  type="info"
                  size="small"
                  round
                  class="test-group-btn"
                >
                  <template #icon>
                    <n-icon><SpeedometerOutline /></n-icon>
                  </template>
                  {{ t('proxy.speedTest') }}
                </n-button>
              </div>
            </div>

            <!-- 节点网格 -->
            <div class="nodes-container">
              <div class="nodes-grid" :style="{ '--grid-columns': gridCols }">
                <div
                  v-for="(proxy, i) in group.all"
                  :key="`${group.name}-${proxy}-${i}`"
                  class="node-card"
                  :class="{
                    'node-active': group.now === proxy,
                    'node-testing': testingNodes[proxy],
                  }"
                >
                  <!-- 节点主要信息 -->
                  <div class="node-header">
                    <div class="node-name">
                      <n-ellipsis :tooltip="{ width: 'trigger' }">
                        {{ proxy }}
                      </n-ellipsis>
                    </div>
                    <div v-if="group.now === proxy" class="active-badge">
                      <n-icon size="12"><CheckmarkCircleOutline /></n-icon>
                    </div>
                  </div>

                  <!-- 延迟信息 -->
                  <div class="node-delay" @click="testSingleNode(proxy)">
                    <div class="delay-display" :class="getNodeStatusType(proxy)">
                      <n-icon v-if="testingNodes[proxy]" size="14" class="loading-icon">
                        <RefreshOutline />
                      </n-icon>
                      <span v-else class="delay-value">{{ getNodeStatusText(proxy) }}</span>
                    </div>
                    <span class="delay-label">{{ t('proxy.delay') }}</span>
                  </div>

                  <!-- 操作按钮 -->
                  <div class="node-footer">
                    <n-button
                      @click="changeProxy(group.name, proxy)"
                      :type="group.now === proxy ? 'success' : 'default'"
                      size="small"
                      :disabled="group.now === proxy"
                      block
                      round
                      class="switch-btn"
                    >
                      <template #icon>
                        <n-icon size="14">
                          <CheckmarkCircleOutline v-if="group.now === proxy" />
                          <SwapHorizontalOutline v-else />
                        </n-icon>
                      </template>
                      {{ group.now === proxy ? t('proxy.inUse') : t('proxy.switch') }}
                    </n-button>
                  </div>

                  <!-- 活跃指示器 -->
                  <div v-if="group.now === proxy" class="active-indicator"></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </n-spin>
    </div>
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
} from '@vicons/ionicons5'
import { useWindowSize } from '@vueuse/core'
import { tauriApi } from '@/services/tauri-api'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores'
import { useThemeStore } from '@/stores/app/ThemeStore'

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
const message = useMessage()
const isLoading = ref(false)
const { width } = useWindowSize()
const { t } = useI18n()
const appStore = useAppStore()
const themeStore = useThemeStore()

// 代理数据
const rawProxies = ref<Record<string, ProxyData>>({})
const proxyGroups = ref<ProxyData[]>([])
const testingNodes = reactive<Record<string, boolean>>({})

// 注册事件监听器
let unlistenTestProgress: (() => void) | null = null
let unlistenTestResult: (() => void) | null = null
let unlistenTestComplete: (() => void) | null = null
let unlistenNodeResult: (() => void) | null = null

// 测试状态和结果
const testingGroup = ref('')
const testResults = reactive<Record<string, number>>({})
const nodeErrors = reactive<Record<string, string>>({})

// 根据窗口宽度调整网格列数
const gridCols = computed(() => {
  if (width.value < 640) return 1
  if (width.value < 960) return 2
  if (width.value < 1280) return 3
  if (width.value < 1600) return 4
  return 5
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
    const data = await tauriApi.proxy.getProxies(appStore.apiPort)
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
    await tauriApi.proxy.testGroupDelay(group, appStore.apiPort)
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
    await tauriApi.proxy.changeProxy(group, proxy, appStore.apiPort)
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
.modern-proxy {
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 24px;
  min-height: 100%;
}

/* 顶部操作栏 */
.proxy-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 32px;
  background: var(--n-card-color);
  border-radius: 20px;
  border: 1px solid var(--n-border-color);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.proxy-header:hover {
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  transform: translateY(-2px);
}

.header-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.page-icon {
  width: 48px;
  height: 48px;
  border-radius: 16px;
  background: linear-gradient(135deg, #646cff, #747bff);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 16px rgba(100, 108, 255, 0.3);
}

.page-title-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
  color: var(--n-text-color);
}

.page-subtitle {
  font-size: 14px;
  color: var(--n-text-color-3);
  margin: 0;
}

.refresh-btn {
  height: 40px;
  min-width: 120px;
}

/* 主要内容区 */
.proxy-main {
  flex: 1;
}

.loading-container {
  min-height: 400px;
}

.loading-text {
  color: var(--n-text-color-2);
  font-size: 14px;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  text-align: center;
}

.empty-icon {
  color: var(--n-text-color-3);
  margin-bottom: 16px;
  opacity: 0.6;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--n-text-color-2);
  margin-bottom: 8px;
}

.empty-description {
  font-size: 14px;
  color: var(--n-text-color-3);
}

/* 代理组样式 */
.proxy-groups {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.group-section {
  background: var(--n-card-color);
  border-radius: 20px;
  border: 1px solid var(--n-border-color);
  overflow: hidden;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.group-section:hover {
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

/* 组头部 */
.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 32px;
  background: rgba(0, 0, 0, 0.02);
  border-bottom: 1px solid var(--n-border-color);
}

.group-title-section {
  display: flex;
  align-items: center;
  gap: 16px;
}

.group-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: var(--n-text-color);
}

.group-meta {
  display: flex;
  gap: 8px;
}

.group-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.current-node-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.current-label {
  font-size: 14px;
  color: var(--n-text-color-2);
  font-weight: 500;
}

.test-group-btn {
  height: 32px;
}

/* 节点容器 */
.nodes-container {
  padding: 24px 32px 32px;
}

.nodes-grid {
  display: grid;
  grid-template-columns: repeat(var(--grid-columns), 1fr);
  gap: 20px;
}

/* 节点卡片 */
.node-card {
  position: relative;
  background: var(--n-card-color);
  border: 1px solid var(--n-border-color);
  border-radius: 16px;
  padding: 20px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.node-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
  border-color: var(--n-primary-color);
}

.node-card.node-active {
  background: rgba(16, 185, 129, 0.06);
  border-color: #10b981;
  box-shadow: 0 4px 16px rgba(16, 185, 129, 0.2);
}

.node-card.node-testing {
  background: rgba(59, 130, 246, 0.06);
  border-color: #3b82f6;
}

/* 节点头部 */
.node-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.node-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color);
  line-height: 1.4;
  flex: 1;
}

.active-badge {
  width: 20px;
  height: 20px;
  border-radius: 10px;
  background: #10b981;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

/* 延迟显示 */
.node-delay {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.node-delay:hover {
  transform: scale(1.02);
}

.delay-display {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 60px;
  height: 32px;
  border-radius: 16px;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.delay-display.success {
  background: rgba(34, 197, 94, 0.15);
  color: #16a34a;
}

.delay-display.info {
  background: rgba(59, 130, 246, 0.15);
  color: #2563eb;
}

.delay-display.warning {
  background: rgba(245, 158, 11, 0.15);
  color: #d97706;
}

.delay-display.error {
  background: rgba(239, 68, 68, 0.15);
  color: #dc2626;
}

.delay-display.default {
  background: rgba(156, 163, 175, 0.15);
  color: #6b7280;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

.delay-label {
  font-size: 12px;
  color: var(--n-text-color-3);
}

/* 节点底部 */
.node-footer {
  margin-top: auto;
}

.switch-btn {
  height: 32px;
  font-size: 12px;
  font-weight: 500;
}

/* 活跃指示器 */
.active-indicator {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, #10b981, #059669);
  border-radius: 16px 16px 0 0;
  animation: pulse 2s ease-in-out infinite;
}

/* 动画 */
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

/* 深色模式适配 */
[data-theme='dark'] .group-header {
  background: rgba(255, 255, 255, 0.02);
}

[data-theme='dark'] .node-card.node-active {
  background: rgba(16, 185, 129, 0.1);
}

[data-theme='dark'] .node-card.node-testing {
  background: rgba(59, 130, 246, 0.1);
}

[data-theme='dark'] .delay-display.success {
  background: rgba(34, 197, 94, 0.2);
  color: #4ade80;
}

[data-theme='dark'] .delay-display.info {
  background: rgba(59, 130, 246, 0.2);
  color: #60a5fa;
}

[data-theme='dark'] .delay-display.warning {
  background: rgba(245, 158, 11, 0.2);
  color: #fcd34d;
}

[data-theme='dark'] .delay-display.error {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

[data-theme='dark'] .delay-display.default {
  background: rgba(156, 163, 175, 0.2);
  color: #d1d5db;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .proxy-header {
    flex-direction: column;
    gap: 16px;
    padding: 20px;
    text-align: center;
  }

  .group-header {
    flex-direction: column;
    gap: 16px;
    padding: 20px;
    text-align: center;
  }

  .group-actions {
    justify-content: center;
  }

  .nodes-container {
    padding: 20px;
  }

  .nodes-grid {
    gap: 16px;
  }

  .node-card {
    padding: 16px;
  }
}

@media (max-width: 480px) {
  .group-title-section {
    flex-direction: column;
    gap: 8px;
  }

  .current-node-info {
    flex-direction: column;
    gap: 4px;
    text-align: center;
  }

  .node-delay {
    flex-direction: column;
    gap: 8px;
  }

  .delay-display {
    width: 80px;
    height: 28px;
  }
}
</style>
