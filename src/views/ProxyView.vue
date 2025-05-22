<template>
  <div class="proxy-container">
    <!-- 顶部标题卡片 -->
    <n-card class="proxy-card" :bordered="false">
      <div class="card-header">
        <div class="header-left">
          <n-h3 class="card-title">
            <n-icon size="22" class="card-icon">
              <swap-horizontal-outline />
            </n-icon>
            {{ t('proxy.title') }}
          </n-h3>
        </div>
        <div class="header-right">
          <!-- 刷新按钮 -->
          <n-tooltip trigger="hover" placement="top">
            <template #trigger>
              <n-button
                quaternary
                circle
                size="small"
                @click="init"
                :loading="isLoading"
                class="refresh-button"
              >
                <template #icon>
                  <n-icon>
                    <refresh-outline />
                  </n-icon>
                </template>
              </n-button>
            </template>
            {{ t('proxy.refreshList') }}
          </n-tooltip>
        </div>
      </div>
    </n-card>

    <!-- 代理列表卡片 -->
    <n-spin :show="isLoading">
      <n-card class="proxy-list-card" :bordered="false">
        <!-- 代理分组内容 -->
        <n-tabs type="line" animated v-model:value="activeGroupTab">
          <n-tab-pane
            v-for="(group, index) in [...proxyGroups].reverse()"
            :key="index"
            :name="group.name"
            :tab="group.name"
          >
            <div class="proxy-group">
              <div class="proxy-group-info">
                <n-space :size="10" wrap-item>
                  <n-tag :bordered="false" type="success" size="small" class="proxy-tag">
                    <template #icon>
                      <n-icon size="14">
                        <checkmark-circle-outline />
                      </n-icon>
                    </template>
                    {{ t('proxy.currentNode') }}: {{ group.now }}
                  </n-tag>
                  <n-tag :bordered="false" type="info" size="small" class="proxy-tag">
                    <template #icon>
                      <n-icon size="14">
                        <layers-outline />
                      </n-icon>
                    </template>
                    {{ group.all.length }} {{ t('proxy.nodeCount') }}
                  </n-tag>
                  <n-tag :bordered="false" type="warning" size="small" class="proxy-tag">
                    <template #icon>
                      <n-icon size="14">
                        <information-circle-outline />
                      </n-icon>
                    </template>
                    {{ group.type }}
                  </n-tag>
                  <n-button
                    @click="testNodeDelay(group.name)"
                    :loading="testingGroup === group.name"
                    size="small"
                    type="info"
                    ghost
                    class="proxy-button"
                  >
                    <template #icon>
                      <n-icon>
                        <speedometer-outline />
                      </n-icon>
                    </template>
                    {{ t('proxy.speedTest') }}
                  </n-button>
                </n-space>
              </div>

              <div class="proxy-grid">
                <div v-for="(proxy, i) in group.all" :key="i" class="proxy-grid-item">
                  <n-card
                    :class="{
                      'proxy-node-card': true,
                      'proxy-node-card-active': group.now === proxy,
                    }"
                    :bordered="false"
                    hoverable
                  >
                    <div class="proxy-card-content">
                      <!-- 节点名称 - 单独占一行 -->
                      <div class="proxy-name-wrapper">
                        <n-ellipsis :tooltip="{ width: 'trigger' }">
                          {{ proxy }}
                        </n-ellipsis>
                      </div>

                      <!-- 底部操作区 -->
                      <div class="proxy-card-footer">
                        <!-- 延迟标签 -->
                        <n-tag
                          :type="getNodeStatusType(proxy)"
                          size="small"
                          :bordered="false"
                          round
                          class="delay-tag"
                          @click="testSingleNode(proxy)"
                          :loading="testingNodes[proxy]"
                          hoverable
                        >
                          {{ getNodeStatusText(proxy) }}
                        </n-tag>

                        <!-- 切换按钮 -->
                        <n-button
                          @click="changeProxy(group.name, proxy)"
                          :type="group.now === proxy ? 'default' : 'primary'"
                          size="small"
                          :disabled="group.now === proxy"
                          :ghost="group.now !== proxy"
                          class="proxy-button switch-button"
                        >
                          <template #icon>
                            <n-icon>
                              <checkmark-circle-outline v-if="group.now === proxy" />
                              <swap-horizontal-outline v-else />
                            </n-icon>
                          </template>
                          {{ group.now === proxy ? t('proxy.inUse') : t('proxy.switch') }}
                        </n-button>
                      </div>
                    </div>
                  </n-card>
                </div>
              </div>
            </div>
          </n-tab-pane>
        </n-tabs>
      </n-card>
    </n-spin>
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
  if (delay === 0) return 'default'
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
  if (delay === 0) return t('proxy.notTested')
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
  max-width: 1200px;
  margin: 0 auto;
  padding: 12px 8px;
  animation: slide-up 0.3s ease;
}

.proxy-card {
  margin-bottom: 12px;
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.proxy-card:hover,
.proxy-list-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08);
}

.proxy-list-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 14px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-weight: 600;
  font-size: 18px;
}

.card-icon {
  color: var(--primary-color);
}

.refresh-button {
  transition: all 0.3s ease;
}

.refresh-button:hover:not(:disabled) {
  transform: translateY(-2px);
  color: var(--primary-color);
  background-color: rgba(64, 128, 255, 0.1);
}

.proxy-group {
  margin-bottom: 16px;
}

.proxy-group-info {
  margin-bottom: 16px;
  padding: 0 2px;
}

.proxy-tag {
  font-weight: 500;
  padding: 0 10px;
  height: 26px;
  border-radius: 13px;
}

.proxy-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 12px;
}

.proxy-node-card {
  transition: all 0.3s ease;
  border-radius: 12px;
  border-left: 3px solid transparent;
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.05);
  background: var(--card-color);
  backdrop-filter: blur(8px);
  overflow: hidden;
  height: 100%;
}

.proxy-node-card :deep(.n-card__content) {
  padding: 12px 14px;
}

.proxy-node-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.1);
}

.proxy-node-card-active {
  border-left: 3px solid var(--success-color);
  background: linear-gradient(135deg, rgba(var(--success-color-rgb), 0.05), transparent);
}

.proxy-card-content {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.proxy-name-wrapper {
  font-weight: 500;
  font-size: 14px;
  color: var(--n-text-color-1);
  margin-bottom: 10px;
  padding: 2px 0;
  line-height: 1.4;
  border-bottom: 1px dashed rgba(128, 128, 128, 0.15);
  padding-bottom: 8px;
}

.proxy-card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: auto;
}

.delay-tag {
  font-weight: 500;
  transition: all 0.2s ease;
  cursor: pointer;
  min-width: 64px;
  text-align: center;
  padding: 0 8px;
  font-size: 12px;
}

.delay-tag:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
}

.proxy-button {
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.switch-button {
  height: 28px;
  font-size: 12px;
}

.proxy-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

:deep(.dark) .proxy-button:hover:not(:disabled) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

:deep(.n-tabs .n-tab-pane) {
  padding: 14px 0;
}

:deep(.n-card.proxy-node-card) {
  background-color: var(--card-color);
}

:deep(.n-card.proxy-node-card:hover) {
  background-color: var(--card-color-hover);
}

:deep(.n-card.proxy-node-card-active) {
  background: linear-gradient(135deg, rgba(var(--success-color-rgb), 0.08), transparent);
}

@keyframes slide-up {
  0% {
    opacity: 0;
    transform: translateY(10px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 640px) {
  .proxy-grid {
    grid-template-columns: 1fr;
  }

  .proxy-tag {
    height: 24px;
    font-size: 12px;
  }

  .switch-button {
    height: 26px;
    font-size: 12px;
  }
}
</style>
