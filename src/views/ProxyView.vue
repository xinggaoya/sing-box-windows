<template>
  <div class="proxy-container">
    <!-- 顶部标题区 -->
    <div class="header-section">
      <div class="header-content">
        <div class="header-left">
          <div class="title-wrapper">
            <div class="title-icon">
              <n-icon size="20">
                <swap-horizontal-outline />
              </n-icon>
            </div>
            <h2 class="page-title">{{ t('proxy.title') }}</h2>
          </div>
        </div>
        <div class="header-actions">
          <n-button
            @click="init"
            :loading="isLoading"
            size="small"
            type="primary"
            ghost
            class="refresh-btn"
            round
          >
            <template #icon>
              <n-icon>
                <refresh-outline />
              </n-icon>
            </template>
            {{ t('proxy.refreshList') }}
          </n-button>
        </div>
      </div>
    </div>

    <!-- 代理内容区 -->
    <div class="proxy-content">
      <n-spin :show="isLoading">
        <n-tabs
          type="card"
          animated
          v-model:value="activeGroupTab"
          class="proxy-tabs"
          tab-style="padding: 8px 16px; font-weight: 500;"
        >
          <n-tab-pane
            v-for="(group, index) in [...proxyGroups].reverse()"
            :key="index"
            :name="group.name"
            class="tab-content"
          >
            <template #tab>
              <div class="tab-header">
                <span>{{ group.name }}</span>
                <n-badge :value="group.all.length" :max="99" show-zero type="info" size="small" />
              </div>
            </template>

            <div class="proxy-group-container">
              <!-- 组信息快览 -->
              <div class="group-overview">
                <div class="overview-cards">
                  <div class="overview-card current-node">
                    <div class="card-icon">
                      <n-icon size="16">
                        <checkmark-circle-outline />
                      </n-icon>
                    </div>
                    <div class="card-content">
                      <div class="card-label">{{ t('proxy.currentNode') }}</div>
                      <div class="card-value">{{ group.now }}</div>
                    </div>
                  </div>

                  <div class="overview-card node-count">
                    <div class="card-icon">
                      <n-icon size="16">
                        <layers-outline />
                      </n-icon>
                    </div>
                    <div class="card-content">
                      <div class="card-label">{{ t('proxy.nodeCount') }}</div>
                      <div class="card-value">{{ group.all.length }}</div>
                    </div>
                  </div>

                  <div class="overview-card group-type">
                    <div class="card-icon">
                      <n-icon size="16">
                        <information-circle-outline />
                      </n-icon>
                    </div>
                    <div class="card-content">
                      <div class="card-label">类型</div>
                      <div class="card-value">{{ group.type }}</div>
                    </div>
                  </div>
                </div>

                <div class="overview-actions">
                  <n-button
                    @click="testNodeDelay(group.name)"
                    :loading="testingGroup === group.name"
                    size="small"
                    type="info"
                    class="test-btn"
                    round
                  >
                    <template #icon>
                      <n-icon>
                        <speedometer-outline />
                      </n-icon>
                    </template>
                    {{ t('proxy.speedTest') }}
                  </n-button>
                </div>
              </div>

              <!-- 代理节点网格 -->
              <div class="nodes-grid">
                <div
                  v-for="(proxy, i) in group.all"
                  :key="i"
                  class="node-card"
                  :class="{ 'node-active': group.now === proxy }"
                >
                  <div class="node-content">
                    <!-- 节点名称 -->
                    <div class="node-name">
                      <n-ellipsis :tooltip="{ width: 'trigger' }">
                        {{ proxy }}
                      </n-ellipsis>
                    </div>

                    <!-- 节点状态和操作 -->
                    <div class="node-footer">
                      <div
                        class="delay-indicator"
                        :class="getNodeStatusType(proxy)"
                        @click="testSingleNode(proxy)"
                      >
                        <n-icon v-if="testingNodes[proxy]" size="12" class="loading-icon">
                          <refresh-outline />
                        </n-icon>
                        <span class="delay-text">{{ getNodeStatusText(proxy) }}</span>
                      </div>

                      <n-button
                        @click="changeProxy(group.name, proxy)"
                        :type="group.now === proxy ? 'success' : 'default'"
                        size="tiny"
                        :disabled="group.now === proxy"
                        class="switch-btn"
                        round
                      >
                        <template #icon>
                          <n-icon size="12">
                            <checkmark-circle-outline v-if="group.now === proxy" />
                            <swap-horizontal-outline v-else />
                          </n-icon>
                        </template>
                        {{ group.now === proxy ? t('proxy.inUse') : t('proxy.switch') }}
                      </n-button>
                    </div>
                  </div>

                  <!-- 活跃指示器 -->
                  <div v-if="group.now === proxy" class="active-indicator"></div>
                </div>
              </div>
            </div>
          </n-tab-pane>
        </n-tabs>
      </n-spin>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { onMounted, ref, computed, reactive, h, onUnmounted } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  CheckmarkCircleOutline,
  SwapHorizontalOutline,
  SpeedometerOutline,
  GlobeOutline,
  LayersOutline,
  HardwareChipOutline,
  ChevronDownOutline,
  InformationCircleOutline,
} from '@vicons/ionicons5'
import { useWindowSize } from '@vueuse/core'
import { Component } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores'

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

// 添加类型定义
interface TestGroupResult {
  group: string
  results: Record<string, number>
  success: boolean
  error?: string
}

interface TestNodeResult {
  proxy: string
  delay?: number
  success: boolean
  error?: string
}

// 状态定义
const message = useMessage()
const isLoading = ref(false)
const { width } = useWindowSize()
const { t } = useI18n()
const appStore = useAppStore()

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

// 动态渲染图标的辅助函数
function renderIcon(icon: Component) {
  return () => h('div', { class: 'dropdown-option-icon' }, h(icon))
}

// 选项卡状态
const activeGroupTab = ref('')

// 根据窗口宽度调整网格列数
const gridCols = computed(() => {
  if (width.value < 640) return 1
  if (width.value < 960) return 2
  if (width.value < 1280) return 3
  return 4
})

// 生命周期钩子
onMounted(() => {
  init()
  // 注册事件监听器
  setupEventListeners()
})

onUnmounted(() => {
  // 清理事件监听器
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

  unlistenTestResult = await listen('test-group-result', (event) => {
    const data = event.payload as TestGroupResult
    if (data.success) {
      // 更新测试结果
      Object.assign(testResults, data.results)
      message.success(t('proxy.groupTestComplete'))
    } else {
      message.error(`${t('proxy.testFailed')}: ${data.error}`)
    }
    testingGroup.value = ''
  })

  unlistenTestComplete = await listen('test-nodes-complete', () => {
    message.success(t('proxy.batchTestComplete'))
  })

  // 添加节点测试结果监听
  unlistenNodeResult = await listen('test-node-result', (event) => {
    const data = event.payload as TestNodeResult
    const { proxy, success, delay, error } = data

    // 取消该节点的加载状态
    testingNodes[proxy] = false

    if (success) {
      if (delay !== undefined) {
        // 更新节点延迟
        testResults[proxy] = delay
        // 清除可能存在的错误
        delete nodeErrors[proxy]
        message.success(`${t('proxy.nodeTestComplete')}: ${proxy}`)
      }
    } else {
      // 记录错误信息
      nodeErrors[proxy] = error || t('proxy.unknownError')
      message.error(`${t('proxy.nodeTestFailed')}: ${proxy}`)
    }
  })
}

/**
 * 初始化并获取代理信息
 */
const init = async () => {
  isLoading.value = true
  try {
    // 使用Tauri API获取代理信息

    const data = await tauriApi.proxy.getProxies(appStore.apiPort)
    rawProxies.value = data.proxies

    // 提取代理组
    const groups: ProxyData[] = []

    Object.entries(data.proxies).forEach(([key, item]) => {
      // 排除特殊组和直连
      if (key === 'GLOBAL' || key === 'direct') return

      // 如果是Selector或URLTest类型，添加到代理组
      if (item.type === 'Selector' || item.type === 'URLTest') {
        groups.push(item)
      }
    })

    proxyGroups.value = groups

    // 默认选中第三个选项卡（注意列表使用了reverse）
    if (groups.length > 0) {
      // 直接设置要选择的标签名称
      if (groups.length >= 3) {
        // 选择倒数第三个组（在UI中会显示为第三个）
        activeGroupTab.value = groups[groups.length - 3].name
      } else {
        // 如果不足三个，选择第一个
        activeGroupTab.value = groups[0].name
      }

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
 * @param name 节点名称
 * @returns 颜色类型
 */
const getNodeStatusType = (name: string): string => {
  // 如果节点有错误，返回错误状态
  if (nodeErrors[name]) return 'error'

  // 如果节点正在测试中
  if (testingNodes[name]) return 'info'

  // 否则根据延迟返回状态
  const delay = testResults[name] || 0
  if (delay === 0) return 'warning' // 没有测试数据，用警告色提示
  if (delay < 100) return 'success'
  if (delay < 200) return 'info'
  if (delay < 300) return 'warning'
  return 'error'
}

/**
 * 获取节点状态文本
 * @param name 节点名称
 * @returns 状态文本
 */
const getNodeStatusText = (name: string): string => {
  // 如果节点正在测试中
  if (testingNodes[name]) return t('proxy.testing')

  // 如果节点有错误
  if (nodeErrors[name]) return t('proxy.timeout')

  // 否则显示延迟
  const delay = testResults[name] || 0
  if (delay === 0) return '--'
  return `${delay}ms`
}

/**
 * 判断是否为真实节点（非组）
 * @param name 节点名称
 * @returns 是否为真实节点
 */
const isRealNode = (name: string): boolean => {
  if (!rawProxies.value[name]) return false
  return !['Selector', 'URLTest', 'Fallback'].includes(rawProxies.value[name].type)
}

/**
 * 测试单个节点延迟
 * @param proxy 节点名称
 */
const testSingleNode = async (proxy: string) => {
  if (testingNodes[proxy]) return

  // 设置测试中状态
  testingNodes[proxy] = true

  try {
    // 清除之前的错误信息
    delete nodeErrors[proxy]

    // 调用后端API测试节点
    await tauriApi.proxy.testNodeDelay(proxy)

    // 注意：此时不设置 testingNodes[proxy] = false
    // 因为这将由事件监听器在收到结果时设置
  } catch (error) {
    console.error(t('proxy.testFailed'), error)
    message.error(t('proxy.testErrorMessage'))
    testingNodes[proxy] = false
    nodeErrors[proxy] = String(error)
  }
}

/**
 * 测试节点延迟
 * @param group 代理组名称
 */
const testNodeDelay = async (group: string) => {
  if (testingGroup.value === group) return

  testingGroup.value = group
  try {
    await tauriApi.proxy.testGroupDelay(group, appStore.apiPort)
  } catch (error) {
    console.error(t('proxy.testFailed'), error)
    message.error(t('proxy.testErrorMessage'))
    testingGroup.value = ''
  }
}

/**
 * 切换代理
 * @param group 代理组名称
 * @param proxy 要切换到的代理名称
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
.proxy-container {
  min-height: calc(100vh - 120px);
  padding: 20px;
  background: linear-gradient(135deg, rgba(64, 128, 255, 0.02), rgba(144, 147, 153, 0.02));
  animation: fadeIn 0.4s ease-out;
}

/* 顶部标题区 */
.header-section {
  margin-bottom: 20px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.04);
  transition: all 0.3s ease;
}

.header-content:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
}

.title-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, #4080ff, #2266dd);
  border-radius: 10px;
  color: white;
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.3);
}

.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color-1);
  background: linear-gradient(135deg, #4080ff, #2266dd);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.refresh-btn {
  height: 36px;
  font-weight: 500;
  transition: all 0.3s ease;
}

.refresh-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(64, 128, 255, 0.2);
}

/* 代理内容区 */
.proxy-content {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(12px);
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.3);
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.06);
  transition: all 0.3s ease;
}

.proxy-content:hover {
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.08);
}

/* 标签页样式 */
.proxy-tabs {
  --n-tab-padding: 12px 20px;
}

.tab-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}

.tab-content {
  padding: 20px;
}

/* 组概览区 */
.group-overview {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
  padding: 16px 20px;
  background: linear-gradient(135deg, rgba(64, 128, 255, 0.05), rgba(144, 147, 153, 0.05));
  border-radius: 16px;
  border: 1px solid rgba(64, 128, 255, 0.1);
}

.overview-cards {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.overview-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.3);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  transition: all 0.2s ease;
  min-width: 120px;
}

.overview-card:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.overview-card .card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  color: white;
}

.current-node .card-icon {
  background: linear-gradient(135deg, #00b42a, #009a1a);
}

.node-count .card-icon {
  background: linear-gradient(135deg, #4080ff, #2266dd);
}

.group-type .card-icon {
  background: linear-gradient(135deg, #ff7d00, #d66600);
}

.card-content {
  flex: 1;
}

.card-label {
  font-size: 11px;
  color: var(--text-color-3);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 2px;
}

.card-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-color-1);
}

.test-btn {
  height: 34px;
  font-weight: 500;
  transition: all 0.3s ease;
}

.test-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.2);
}

/* 节点网格 */
.nodes-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 20px;
  padding: 10px;
}

.node-card {
  position: relative;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  backdrop-filter: blur(8px);
  min-height: 95px;
  display: flex;
  flex-direction: column;
}

.node-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.08);
  border-color: rgba(64, 128, 255, 0.2);
}

.node-card.node-active {
  background: linear-gradient(135deg, rgba(0, 180, 42, 0.08), rgba(0, 154, 26, 0.08));
  border-color: rgba(0, 180, 42, 0.3);
  box-shadow: 0 6px 18px rgba(0, 180, 42, 0.12);
}

.node-card.node-active:hover {
  box-shadow: 0 8px 24px rgba(0, 180, 42, 0.16);
}

.node-content {
  padding: 12px;
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 0;
}

.node-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-color-1);
  margin-bottom: 8px;
  line-height: 1.3;
  word-break: break-word;
  overflow-wrap: break-word;
  flex: 1;
  display: flex;
  align-items: flex-start;
}

.node-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  flex-shrink: 0;
  margin-top: auto;
}

.delay-indicator {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 4px 6px;
  border-radius: 6px;
  font-size: 10px;
  font-weight: 600;
  transition: all 0.2s ease;
  cursor: pointer;
  min-width: 45px;
  justify-content: center;
  height: 24px;
}

.delay-indicator:hover {
  transform: scale(1.05);
}

.delay-indicator.success {
  background: linear-gradient(135deg, rgba(0, 180, 42, 0.15), rgba(0, 154, 26, 0.15));
  color: #009a1a;
}

.delay-indicator.info {
  background: linear-gradient(135deg, rgba(144, 147, 153, 0.15), rgba(123, 126, 131, 0.15));
  color: #7b7e83;
}

.delay-indicator.warning {
  background: linear-gradient(135deg, rgba(255, 125, 0, 0.15), rgba(214, 102, 0, 0.15));
  color: #d66600;
}

.delay-indicator.error {
  background: linear-gradient(135deg, rgba(245, 63, 63, 0.15), rgba(203, 42, 42, 0.15));
  color: #cb2a2a;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

.delay-text {
  font-size: 10px;
  font-weight: 600;
}

.switch-btn {
  height: 24px;
  font-size: 10px;
  font-weight: 500;
  transition: all 0.2s ease;
  min-width: 50px;
  flex-shrink: 0;
  padding: 0 8px;
}

.switch-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 3px 6px rgba(0, 0, 0, 0.08);
}

/* 活跃指示器 */
.active-indicator {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 3px;
  background: linear-gradient(90deg, #00b42a, #009a1a);
  animation: pulse 2s ease-in-out infinite;
}

/* 暗黑模式适配 */
:deep(.dark) .header-content {
  background: rgba(24, 24, 28, 0.8);
  border-color: rgba(255, 255, 255, 0.1);
}

:deep(.dark) .proxy-content {
  background: rgba(24, 24, 28, 0.9);
  border-color: rgba(255, 255, 255, 0.1);
}

:deep(.dark) .overview-card {
  background: rgba(40, 40, 48, 0.8);
  border-color: rgba(255, 255, 255, 0.1);
}

:deep(.dark) .node-card {
  background: rgba(40, 40, 48, 0.9);
  border-color: rgba(255, 255, 255, 0.1);
}

:deep(.dark) .node-card:hover {
  border-color: rgba(64, 128, 255, 0.3);
}

:deep(.dark) .group-overview {
  background: linear-gradient(135deg, rgba(64, 128, 255, 0.08), rgba(144, 147, 153, 0.08));
  border-color: rgba(64, 128, 255, 0.15);
}

/* 动画效果 */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
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
    opacity: 0.5;
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .proxy-container {
    padding: 12px;
  }

  .header-content {
    padding: 12px 16px;
    flex-direction: column;
    gap: 12px;
  }

  .title-wrapper {
    justify-content: center;
  }

  .group-overview {
    flex-direction: column;
    gap: 16px;
    align-items: stretch;
  }

  .overview-cards {
    justify-content: center;
  }

  .overview-actions {
    align-self: center;
  }

  .nodes-grid {
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 16px;
  }

  .node-card {
    min-height: 85px;
  }

  .tab-content {
    padding: 16px 12px;
  }
}

@media (max-width: 480px) {
  .overview-cards {
    flex-direction: column;
  }

  .overview-card {
    min-width: auto;
  }

  .nodes-grid {
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 12px;
  }

  .node-card {
    min-height: 80px;
  }

  .node-content {
    padding: 10px;
  }

  .node-name {
    font-size: 12px;
    margin-bottom: 6px;
  }

  .node-footer {
    flex-direction: column;
    gap: 6px;
    align-items: stretch;
  }

  .delay-indicator {
    align-self: center;
    min-width: 60px;
    height: 22px;
    font-size: 9px;
  }

  .switch-btn {
    width: 100%;
    min-width: auto;
    height: 22px;
    font-size: 9px;
  }
}

@media (min-width: 1200px) {
  .nodes-grid {
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 24px;
  }
}

@media (min-width: 1600px) {
  .nodes-grid {
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 28px;
  }
}

/* Naive UI 组件样式覆盖 */
:deep(.n-tabs .n-tabs-nav) {
  background: rgba(248, 250, 252, 0.8);
  backdrop-filter: blur(8px);
}

:deep(.n-tabs .n-tab-pane) {
  padding: 0;
}

:deep(.n-spin-container) {
  min-height: 200px;
}

:deep(.n-badge) {
  --n-font-size: 10px;
}
</style>
