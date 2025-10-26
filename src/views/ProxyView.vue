<template>
  <div class="modern-proxy">
    <!-- 工具栏区域 -->
    <div class="toolbar-section">
      <n-card class="toolbar-card" :bordered="false">
        <div class="toolbar-content">
          <div class="toolbar-left">
            <div class="toolbar-icon">
              <n-icon size="20"><SwapHorizontalOutline /></n-icon>
            </div>
            <div class="toolbar-info">
              <h2 class="toolbar-title">{{ t('proxy.title') }}</h2>
              <p class="toolbar-subtitle">{{ proxyGroups.length }} {{ t('proxy.nodeCount') }}</p>
            </div>
          </div>

          <div class="toolbar-actions">
            <n-button
              @click="init"
              :loading="isLoading"
              type="primary"
              size="medium"
              class="refresh-btn"
            >
              <template #icon>
                <n-icon size="16"><RefreshOutline /></n-icon>
              </template>
              {{ t('common.refresh') }}
            </n-button>
          </div>
        </div>
      </n-card>
    </div>

    <!-- 主内容区域 -->
    <div class="content-section">
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
  ChevronDownOutline,
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
defineOptions({
  name: 'ProxyView'
})

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
.modern-proxy {
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 20px;
  min-height: 100%;
  animation: fadeIn 0.4s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* 工具栏区域 */
.toolbar-section {
  margin-bottom: 4px;
}

.toolbar-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.9)"');
  backdrop-filter: blur(12px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  transition: all 0.3s ease;
}

.toolbar-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 32px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.08)"');
}

.toolbar-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 4px;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
}

.toolbar-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: linear-gradient(135deg, #5b4cfd, #7c3aed);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
  box-shadow: 0 4px 12px rgba(91, 76, 253, 0.3);
}

.toolbar-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.toolbar-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--n-text-color-1);
  margin: 0;
  letter-spacing: -0.02em;
}

.toolbar-subtitle {
  font-size: 13px;
  color: var(--n-text-color-3);
  margin: 0;
}

.toolbar-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.refresh-btn {
  min-width: 100px;
  height: 40px;
  font-weight: 600;
  border-radius: 10px;
}

/* 主内容区域 */
.content-section {
  flex: 1;
}

.loading-container {
  min-height: 300px;
}

.loading-text {
  color: var(--n-text-color-3);
  font-size: 14px;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 16px;
  text-align: center;
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.9)"');
  backdrop-filter: blur(12px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  border-radius: 16px;
  margin: 0 4px;
}

.empty-icon {
  color: var(--n-text-color-disabled);
  margin-bottom: 12px;
  opacity: 0.6;
}

.empty-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0 0 8px 0;
}

.empty-description {
  font-size: 14px;
  color: var(--n-text-color-3);
  margin: 0 0 24px 0;
  line-height: 1.5;
  max-width: 300px;
}

/* 代理组列表 */
.proxy-groups {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.proxy-group {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.9)"');
  backdrop-filter: blur(12px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  border-radius: 16px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.proxy-group:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 32px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.08)"');
}

/* 组头部 */
.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  cursor: pointer;
  user-select: none;
  transition: all 0.2s ease;
}

.group-header:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
}

.group-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.group-main {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.group-name {
  font-size: 18px;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0;
}

.group-badges {
  display: flex;
  gap: 8px;
}

.group-current {
  display: flex;
  align-items: center;
  gap: 8px;
}

.current-label {
  font-size: 13px;
  color: var(--n-text-color-3);
  font-weight: 500;
}

.group-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.test-btn {
  height: 32px;
  min-width: 80px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 8px;
}

.expand-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  transition: all 0.2s ease;
  color: var(--n-text-color-2);
}

.expand-icon.expanded {
  transform: rotate(180deg);
}

.expand-icon:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.12)" : "rgba(0, 0, 0, 0.08)"');
  color: var(--n-text-color-1);
}

/* 节点容器 */
.nodes-container {
  padding: 0 16px 16px;
}

.nodes-grid {
  display: grid;
  grid-template-columns: repeat(var(--grid-columns), 1fr);
  gap: 12px;
}

/* 节点卡片 */
.node-card {
  position: relative;
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  border-radius: 12px;
  padding: 16px;
  min-height: 72px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 12px;
  overflow: hidden;
}

.node-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.08)"');
  border-color: #5b4cfd;
}

.node-card.node-active {
  background: v-bind('themeStore.isDark ? "rgba(16, 185, 129, 0.15)" : "rgba(16, 185, 129, 0.1)"');
  border-color: #10b981;
}

.node-card.node-testing {
  background: v-bind('themeStore.isDark ? "rgba(59, 130, 246, 0.15)" : "rgba(59, 130, 246, 0.1)"');
  border-color: #3b82f6;
}

/* 节点状态指示器 */
.node-status {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.node-status.success {
  background: rgba(34, 197, 94, 0.2);
  color: #10b981;
}

.node-status.info {
  background: rgba(59, 130, 246, 0.2);
  color: #2563eb;
}

.node-status.warning {
  background: rgba(245, 158, 11, 0.2);
  color: #d97706;
}

.node-status.error {
  background: rgba(239, 68, 68, 0.2);
  color: #dc2626;
}

.node-status.default {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.1)" : "rgba(0, 0, 0, 0.1)"');
  color: var(--n-text-color-3);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 节点信息 */
.node-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.node-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.node-delay {
  cursor: pointer;
  transition: all 0.15s ease;
}

.node-delay:hover {
  transform: scale(1.05);
}

.delay-value {
  font-size: 12px;
  font-weight: 600;
  color: var(--n-text-color-2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 活跃指示线 */
.active-indicator {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: linear-gradient(180deg, #10b981, #059669);
  border-radius: 0 2px 2px 0;
}

/* 动画 */
.group-expand-enter-active,
.group-expand-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.group-expand-enter-from,
.group-expand-leave-to {
  max-height: 0;
  opacity: 0;
  transform: translateY(-10px);
}

.group-expand-enter-to,
.group-expand-leave-from {
  max-height: 1000px;
  opacity: 1;
  transform: translateY(0);
}

/* 响应式设计 */
@media (max-width: 1024px) {
  .nodes-grid {
    gap: 10px;
  }

  .node-card {
    min-height: 68px;
    padding: 14px;
  }
}

@media (max-width: 768px) {
  .modern-proxy {
    gap: 16px;
  }

  .toolbar-content {
    flex-direction: column;
    gap: 16px;
    text-align: center;
  }

  .toolbar-left {
    flex-direction: column;
    gap: 12px;
  }

  .group-header {
    padding: 16px;
  }

  .group-main {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .group-current {
    align-self: flex-start;
  }

  .nodes-container {
    padding: 0 16px 16px;
  }

  .nodes-grid {
    gap: 8px;
  }

  .node-card {
    min-height: 64px;
    padding: 12px;
  }

  .node-name {
    font-size: 13px;
  }

  .delay-value {
    font-size: 11px;
  }
}

@media (max-width: 480px) {
  .group-actions {
    gap: 8px;
  }

  .test-btn {
    min-width: 70px;
    font-size: 12px;
  }

  .expand-icon {
    width: 28px;
    height: 28px;
  }

  .node-card {
    min-height: 60px;
    padding: 10px;
  }

  .node-status {
    width: 20px;
    height: 20px;
  }

  .node-name {
    font-size: 12px;
  }

  .delay-value {
    font-size: 10px;
  }
}

/* 优化Naive UI组件 */
:deep(.n-card) {
  border-radius: 16px;
}

:deep(.n-card__content) {
  padding: 24px;
}

:deep(.n-button) {
  border-radius: 10px;
  font-weight: 500;
}

:deep(.n-tag) {
  font-weight: 500;
}

:deep(.n-spin-container) {
  min-height: 300px;
}
</style>