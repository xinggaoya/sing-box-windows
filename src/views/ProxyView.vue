<template>
  <div class="proxy-container">
    <!-- 顶部标题卡片 -->
    <n-card class="proxy-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <n-h3 class="card-title">
              <n-icon size="24" class="card-icon">
                <swap-horizontal-outline />
              </n-icon>
              代理设置
            </n-h3>
          </div>
          <div class="header-right">
            <!-- 代理模式切换 -->
            <n-dropdown :options="proxyModeOptions" @select="handleProxyModeChange" trigger="click">
              <n-tooltip trigger="hover" placement="top">
                <template #trigger>
                  <n-tag class="mode-tag" :bordered="false" type="success" size="medium" hoverable>
                    <n-icon size="18" class="mode-icon">
                      <globe-outline v-if="currentProxyMode === 'global'" />
                      <layers-outline v-if="currentProxyMode === 'rule'" />
                      <hardware-chip-outline v-if="currentProxyMode === 'tun'" />
                    </n-icon>
                    {{ getProxyModeText(currentProxyMode) }}
                    <n-icon size="16" class="dropdown-icon">
                      <chevron-down-outline />
                    </n-icon>
                  </n-tag>
                </template>
                点击切换代理模式
              </n-tooltip>
            </n-dropdown>

            <!-- 刷新按钮 -->
            <n-tooltip trigger="hover" placement="top">
              <template #trigger>
                <n-button
                  quaternary
                  circle
                  size="medium"
                  @click="init"
                  :loading="isLoading"
                  class="refresh-button"
                >
                  <template #icon>
                    <n-icon><refresh-outline /></n-icon>
                  </template>
                </n-button>
              </template>
              刷新代理列表
            </n-tooltip>
          </div>
        </div>
      </template>
    </n-card>

    <!-- 代理模式切换对话框 -->
    <n-modal
      v-model:show="showModeChangeModal"
      preset="dialog"
      :title="`切换到${targetProxyMode ? getProxyModeText(targetProxyMode) : ''}`"
    >
      <template #header>
        <div class="modal-header">
          <n-icon size="22" class="modal-icon">
            <information-circle-outline />
          </n-icon>
          <span>切换到{{ targetProxyMode ? getProxyModeText(targetProxyMode) : '' }}</span>
        </div>
      </template>
      <div class="modal-content">切换代理模式需要重启内核才能生效。确定要切换并重启内核吗？</div>
      <template #action>
        <div class="modal-footer">
          <n-space justify="end">
            <n-button @click="showModeChangeModal = false">取消</n-button>
            <n-button type="primary" :loading="isChangingMode" @click="confirmProxyModeChange">
              确认切换
            </n-button>
          </n-space>
        </div>
      </template>
    </n-modal>

    <!-- 代理列表卡片 -->
    <n-spin :show="isLoading">
      <n-card class="proxy-list-card" :bordered="false">
        <n-tabs type="segment" animated class="proxy-tabs" v-model:value="activeTab">
          <!-- 全局设置选项卡 -->
          <n-tab-pane name="global" tab="全局设置">
            <div v-if="globalGroup" class="proxy-group">
              <div class="proxy-group-info">
                <n-space align="center" :size="12">
                  <n-tag :bordered="false" type="success" size="medium" class="proxy-tag">
                    当前使用: {{ globalGroup.now }}
                  </n-tag>
                  <n-tag :bordered="false" type="info" size="medium" class="proxy-tag">
                    {{ globalGroup.all.length }} 个可选项
                  </n-tag>
                </n-space>
              </div>

              <n-grid :x-gap="16" :y-gap="16" :cols="gridCols" responsive="screen">
                <n-grid-item v-for="(proxy, i) in globalGroup.all" :key="i">
                  <n-card
                    :class="{
                      'proxy-node-card': true,
                      'proxy-node-card-active': globalGroup.now === proxy,
                    }"
                    :bordered="false"
                    hoverable
                  >
                    <n-space vertical :size="14">
                      <n-flex justify="space-between" align="center">
                        <div class="proxy-name-container">
                          <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                            {{ proxy }}
                          </n-ellipsis>
                        </div>
                        <n-tag
                          :type="getProxyTypeColor(getProxyType(proxy))"
                          size="small"
                          :bordered="false"
                          round
                          class="type-tag"
                        >
                          {{ getProxyType(proxy) }}
                        </n-tag>
                      </n-flex>

                      <n-flex justify="space-between" align="center">
                        <n-button
                          @click="changeProxy('GLOBAL', proxy)"
                          :type="globalGroup.now === proxy ? 'default' : 'primary'"
                          size="small"
                          :disabled="globalGroup.now === proxy"
                          :ghost="globalGroup.now !== proxy"
                          class="proxy-button"
                        >
                          <template #icon>
                            <n-icon>
                              <checkmark-circle-outline v-if="globalGroup.now === proxy" />
                              <swap-horizontal-outline v-else />
                            </n-icon>
                          </template>
                          {{ globalGroup.now === proxy ? '使用中' : '切换' }}
                        </n-button>
                      </n-flex>
                    </n-space>
                  </n-card>
                </n-grid-item>
              </n-grid>
            </div>
          </n-tab-pane>

          <!-- 分组选项卡 -->
          <n-tab-pane name="groups" tab="代理分组">
            <n-tabs type="line" animated v-model:value="activeGroupTab">
              <n-tab-pane
                v-for="(group, index) in proxyGroups"
                :key="index"
                :name="group.name"
                :tab="group.name"
              >
                <div class="proxy-group">
                  <div class="proxy-group-info">
                    <n-space align="center" :size="12">
                      <n-tag :bordered="false" type="success" size="medium" class="proxy-tag">
                        当前节点: {{ group.now }}
                      </n-tag>
                      <n-tag :bordered="false" type="info" size="medium" class="proxy-tag">
                        {{ group.all.length }} 个节点
                      </n-tag>
                      <n-tag :bordered="false" type="warning" size="medium" class="proxy-tag">
                        {{ group.type }}
                      </n-tag>
                    </n-space>
                  </div>

                  <n-grid :x-gap="16" :y-gap="16" :cols="gridCols" responsive="screen">
                    <n-grid-item v-for="(proxy, i) in group.all" :key="i">
                      <n-card
                        :class="{
                          'proxy-node-card': true,
                          'proxy-node-card-active': group.now === proxy,
                        }"
                        :bordered="false"
                        hoverable
                      >
                        <n-space vertical :size="14">
                          <n-flex justify="space-between" align="center">
                            <div class="proxy-name-container">
                              <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                                {{ proxy }}
                              </n-ellipsis>
                            </div>
                            <n-tag
                              :type="getNodeDelayType(getNodeDelay(proxy))"
                              size="small"
                              :bordered="false"
                              round
                              class="delay-tag"
                            >
                              {{
                                getNodeDelay(proxy) === 0 ? '未测速' : getNodeDelay(proxy) + 'ms'
                              }}
                            </n-tag>
                          </n-flex>

                          <n-flex justify="space-between" align="center">
                            <n-button
                              @click="changeProxy(group.name, proxy)"
                              :type="group.now === proxy ? 'default' : 'primary'"
                              size="small"
                              :disabled="group.now === proxy"
                              :ghost="group.now !== proxy"
                              class="proxy-button"
                            >
                              <template #icon>
                                <n-icon>
                                  <checkmark-circle-outline v-if="group.now === proxy" />
                                  <swap-horizontal-outline v-else />
                                </n-icon>
                              </template>
                              {{ group.now === proxy ? '使用中' : '切换' }}
                            </n-button>
                            <n-button
                              @click="testNodeDelay(proxy)"
                              :loading="testingNodes[proxy]"
                              secondary
                              size="small"
                              type="info"
                              ghost
                              class="proxy-button"
                              v-if="isRealNode(proxy)"
                            >
                              <template #icon>
                                <n-icon><speedometer-outline /></n-icon>
                              </template>
                              测速
                            </n-button>
                          </n-flex>
                        </n-space>
                      </n-card>
                    </n-grid-item>
                  </n-grid>
                </div>
              </n-tab-pane>
            </n-tabs>
          </n-tab-pane>

          <!-- 节点选项卡 -->
          <n-tab-pane name="nodes" tab="所有节点">
            <div class="nodes-filter">
              <n-space :size="16" align="center">
                <n-input
                  v-model:value="searchText"
                  placeholder="搜索节点名称..."
                  clearable
                  class="search-input"
                >
                  <template #prefix>
                    <n-icon><search-outline /></n-icon>
                  </template>
                </n-input>

                <n-button
                  @click="batchTestAllNodes"
                  :loading="batchTesting"
                  type="primary"
                  ghost
                  size="medium"
                >
                  <template #icon>
                    <n-icon><flash-outline /></n-icon>
                  </template>
                  批量测速
                </n-button>
              </n-space>

              <!-- 批量测试进度条 -->
              <n-progress
                v-if="batchTesting"
                type="line"
                :percentage="batchTestProgress.percentage"
                :indicator-placement="'inside'"
                class="batch-test-progress"
              >
                {{ batchTestProgress.text }}
              </n-progress>
            </div>

            <n-grid :x-gap="16" :y-gap="16" :cols="gridCols" responsive="screen">
              <n-grid-item v-for="(node, i) in filteredNodes" :key="i">
                <n-card class="proxy-node-card node-card" :bordered="false" hoverable>
                  <n-space vertical :size="14">
                    <n-flex justify="space-between" align="center">
                      <div class="proxy-name-container">
                        <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                          {{ node.name }}
                        </n-ellipsis>
                      </div>
                      <n-tag
                        :type="getNodeDelayType(node.delay)"
                        size="small"
                        :bordered="false"
                        round
                        class="delay-tag"
                      >
                        {{ node.delay === 0 ? '未测速' : node.delay + 'ms' }}
                      </n-tag>
                    </n-flex>

                    <n-flex align="center" class="node-type">
                      <n-tag size="small" :bordered="false" :type="getProxyTypeColor(node.type)">
                        {{ node.type }}
                      </n-tag>
                    </n-flex>

                    <n-flex justify="center" align="center">
                      <n-button
                        @click="testNodeDelay(node.name)"
                        :loading="testingNodes[node.name]"
                        secondary
                        size="small"
                        type="info"
                        ghost
                        class="proxy-button"
                      >
                        <template #icon>
                          <n-icon><speedometer-outline /></n-icon>
                        </template>
                        测速
                      </n-button>
                    </n-flex>
                  </n-space>
                </n-card>
              </n-grid-item>
            </n-grid>

            <n-empty
              v-if="filteredNodes.length === 0"
              description="未找到节点"
              class="empty-container"
            />
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
  SearchOutline,
  FlashOutline,
  ChevronDownOutline,
  InformationCircleOutline,
} from '@vicons/ionicons5'
import { useWindowSize } from '@vueuse/core'
import { Component } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { listen } from '@tauri-apps/api/event'

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

interface NodeInfo {
  name: string
  type: string
  delay: number
}

// 状态定义
const message = useMessage()
const isLoading = ref(false)
const { width } = useWindowSize()

// 代理数据
const rawProxies = ref<Record<string, ProxyData>>({})
const globalGroup = ref<ProxyData | null>(null)
const proxyGroups = ref<ProxyData[]>([])
const allNodes = ref<NodeInfo[]>([])
const testingNodes = reactive<Record<string, boolean>>({})
const currentProxyMode = ref('rule') // 默认为规则模式

// 代理模式切换相关
const isChangingMode = ref(false)
const showModeChangeModal = ref(false)
const targetProxyMode = ref('')

// 批量测试相关
const batchTesting = ref(false)
const batchTestProgress = reactive({
  percentage: 0,
  text: '准备测试...',
  current: 0,
  total: 0,
})

// 注册事件监听器
let unlistenTestProgress: (() => void) | null = null
let unlistenTestResult: (() => void) | null = null
let unlistenTestComplete: (() => void) | null = null

// 代理模式选项
const proxyModeOptions = [
  {
    label: '全局模式',
    key: 'global',
    icon: renderIcon(GlobeOutline),
  },
  {
    label: '规则模式',
    key: 'rule',
    icon: renderIcon(LayersOutline),
  },
]

// 动态渲染图标的辅助函数
function renderIcon(icon: Component) {
  return () => h('div', { class: 'dropdown-option-icon' }, h(icon))
}

// 选项卡状态
const activeTab = ref('global')
const activeGroupTab = ref('')
const searchText = ref('')

// 根据窗口宽度调整网格列数
const gridCols = computed(() => {
  if (width.value < 640) return 1
  if (width.value < 960) return 2
  if (width.value < 1280) return 3
  return 4
})

// 过滤节点
const filteredNodes = computed(() => {
  if (!searchText.value) return allNodes.value
  const keyword = searchText.value.toLowerCase()
  return allNodes.value.filter((node) => node.name.toLowerCase().includes(keyword))
})

// 生命周期钩子
onMounted(() => {
  init()
  // 读取当前代理模式
  getCurrentProxyMode()
  // 注册事件监听器
  setupEventListeners()
})

onUnmounted(() => {
  // 清理事件监听器
  if (unlistenTestProgress) unlistenTestProgress()
  if (unlistenTestResult) unlistenTestResult()
  if (unlistenTestComplete) unlistenTestComplete()
})

// 设置事件监听器
const setupEventListeners = async () => {
  unlistenTestProgress = await listen('test-nodes-progress', (event) => {
    const data = event.payload as any
    batchTestProgress.current = data.current
    batchTestProgress.total = data.total
    batchTestProgress.percentage = (data.current / data.total) * 100
    batchTestProgress.text = `测试中: ${data.current}/${data.total} (${data.node})`
  })

  unlistenTestResult = await listen('test-node-result', (event) => {
    const data = event.payload as any
    // 更新节点延迟信息
    const nodeIndex = allNodes.value.findIndex((node) => node.name === data.name)
    if (nodeIndex !== -1) {
      allNodes.value[nodeIndex].delay = data.success ? data.delay : 0
    }
    // 对于失败的情况，可以考虑显示错误信息
    if (!data.success) {
      console.warn(`测试节点 ${data.name} 失败: ${data.error}`)
    }
  })

  unlistenTestComplete = await listen('test-nodes-complete', () => {
    batchTesting.value = false
    message.success('批量测速完成')
  })
}

/**
 * 初始化并获取代理信息
 */
const init = async () => {
  isLoading.value = true
  try {
    // 使用Tauri API获取代理信息
    const data = await tauriApi.proxy.getProxies()
    rawProxies.value = data.proxies

    // 提取全局组
    if (data.proxies.GLOBAL) {
      globalGroup.value = data.proxies.GLOBAL
    }

    // 提取代理组
    const groups: ProxyData[] = []
    const nodes: NodeInfo[] = []

    Object.entries(data.proxies).forEach(([key, item]) => {
      // 排除特殊组和直连
      if (key === 'GLOBAL' || key === 'direct') return

      // 如果是Selector或URLTest类型，添加到代理组
      if (item.type === 'Selector' || item.type === 'URLTest') {
        groups.push(item)

        // 如果还没有设置活动组，设置第一个找到的组为活动组
        if (!activeGroupTab.value && item.type === 'Selector') {
          activeGroupTab.value = item.name
        }
      }

      // 如果不是组类型，添加到节点列表
      if (!['Selector', 'URLTest', 'Fallback'].includes(item.type)) {
        const delay = item.history.length > 0 ? item.history[0].delay : 0
        nodes.push({
          name: item.name,
          type: item.type,
          delay,
        })
      }
    })

    proxyGroups.value = groups
    allNodes.value = nodes

    if (groups.length > 0 || nodes.length > 0) {
      message.success('代理列表加载成功')
    }
  } catch (error) {
    console.error('获取代理列表失败', error)
    message.error('获取代理列表失败，请检查Sing-Box是否已启动')
  } finally {
    isLoading.value = false
  }
}

/**
 * 获取节点类型
 * @param name 节点名称
 * @returns 节点类型
 */
const getProxyType = (name: string): string => {
  if (rawProxies.value[name]) {
    return rawProxies.value[name].type
  }
  return '未知'
}

/**
 * 获取节点类型对应的颜色
 * @param type 节点类型
 * @returns 颜色类型
 */
const getProxyTypeColor = (type: string): string => {
  const typeMap: Record<string, string> = {
    Selector: 'info',
    URLTest: 'success',
    Fallback: 'warning',
    Direct: 'default',
    Hysteria2: 'error',
    Shadowsocks: 'warning',
    Trojan: 'primary',
    VMess: 'info',
    Socks5: 'default',
  }
  return typeMap[type] || 'default'
}

/**
 * 获取节点的延迟
 * @param name 节点名称
 * @returns 延迟值（毫秒）
 */
const getNodeDelay = (name: string): number => {
  if (rawProxies.value[name] && rawProxies.value[name].history.length > 0) {
    return rawProxies.value[name].history[0].delay
  }
  return 0
}

/**
 * 获取延迟对应的颜色类型
 * @param delay 延迟（毫秒）
 * @returns 颜色类型
 */
const getNodeDelayType = (delay: number): string => {
  if (delay === 0) return 'default'
  if (delay < 100) return 'success'
  if (delay < 200) return 'info'
  if (delay < 300) return 'warning'
  return 'error'
}

/**
 * 获取代理模式对应的文本
 * @param mode 代理模式
 * @returns 模式文本
 */
const getProxyModeText = (mode: string): string => {
  const modeMap: Record<string, string> = {
    global: '全局模式',
    rule: '规则模式',
    tun: 'TUN模式',
  }
  return modeMap[mode] || '未知模式'
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
 * 测试节点延迟
 * @param name 节点名称
 * @param server 测试服务器URL
 */
const testNodeDelay = async (
  name: string,
  server: string = 'https://www.gstatic.com/generate_204',
) => {
  if (!rawProxies.value[name]) return

  // 设置正在测试状态
  testingNodes[name] = true

  try {
    const data = await tauriApi.proxy.testNodeDelay(name, server)

    // 更新节点的延迟信息
    const nodeIndex = allNodes.value.findIndex((node) => node.name === name)
    if (nodeIndex !== -1) {
      allNodes.value[nodeIndex].delay = data.delay
    }

    message.success(`测速完成: ${data.delay}ms`)
  } catch (error) {
    console.error('测速失败', error)
    message.error('测速失败，可能是节点无法连接或API未响应')
  } finally {
    // 清除测试状态
    testingNodes[name] = false
  }
}

/**
 * 批量测试所有节点
 */
const batchTestAllNodes = async () => {
  if (batchTesting.value) return

  // 筛选出要测试的节点
  const nodesToTest = allNodes.value
    .filter((node) => !['Direct', 'Reject'].includes(node.type))
    .map((node) => node.name)

  if (nodesToTest.length === 0) {
    message.warning('没有可测试的节点')
    return
  }

  // 重置进度
  batchTesting.value = true
  batchTestProgress.current = 0
  batchTestProgress.total = nodesToTest.length
  batchTestProgress.percentage = 0
  batchTestProgress.text = '准备测试...'

  try {
    // 调用后端API进行批量测试
    await tauriApi.proxy.batchTestNodes(nodesToTest)
  } catch (error) {
    console.error('批量测试失败', error)
    message.error('批量测试失败: ' + error)
    batchTesting.value = false
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
    message.success(`已切换 ${group} 到 ${proxy}`)
    // 重新加载数据
    await init()
  } catch (error) {
    console.error('切换失败', error)
    message.error('切换失败，请检查Sing-Box是否已启动')
  }
}

/**
 * 获取当前代理模式
 */
const getCurrentProxyMode = async () => {
  try {
    // 调用后端API获取当前代理模式
    const mode = await tauriApi.proxy.getCurrentProxyMode()
    currentProxyMode.value = mode
    console.log('当前代理模式:', mode)
  } catch (error) {
    console.error('获取代理模式失败', error)
    // 出错时仍使用默认的规则模式
    currentProxyMode.value = 'rule'
  }
}

/**
 * 处理代理模式变更
 */
const handleProxyModeChange = (key: string) => {
  if (key === currentProxyMode.value) return

  targetProxyMode.value = key
  showModeChangeModal.value = true
}

/**
 * 确认切换代理模式
 */
const confirmProxyModeChange = async () => {
  if (!targetProxyMode.value) return

  isChangingMode.value = true
  try {
    // 调用后端API切换代理模式
    await tauriApi.proxy.toggleProxyMode(targetProxyMode.value)

    // 重启内核以应用更改
    await tauriApi.kernel.restartKernel()

    // 更新当前模式
    currentProxyMode.value = targetProxyMode.value
    message.success(`已切换到${getProxyModeText(targetProxyMode.value)}并重启内核`)

    // 重新加载代理信息
    await init()
  } catch (error) {
    console.error('切换代理模式失败', error)
    message.error(`切换代理模式失败: ${error}`)
  } finally {
    isChangingMode.value = false
    showModeChangeModal.value = false
  }
}
</script>

<style scoped>
.proxy-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 8px;
  animation: slide-up 0.4s ease;
}

.proxy-card {
  margin-bottom: 16px;
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.proxy-card:hover,
.proxy-list-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.proxy-list-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-weight: 600;
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

.mode-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
}

.mode-icon {
  margin-right: 4px;
}

.dropdown-icon {
  margin-left: 4px;
}

.dropdown-option-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  margin-right: 8px;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

.modal-icon {
  color: var(--primary-color);
}

.modal-content {
  margin: 16px 0;
  line-height: 1.6;
}

.modal-footer {
  margin-top: 8px;
}

.proxy-group {
  margin-bottom: 20px;
}

.proxy-group-info {
  margin-bottom: 20px;
  padding: 0 4px;
}

.proxy-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
}

.proxy-node-card {
  transition: all 0.3s ease;
  border-radius: 12px;
  border-left: 3px solid transparent;
}

.proxy-node-card:hover {
  transform: translateY(-3px);
  box-shadow: var(--shadow-medium);
}

.proxy-node-card-active {
  border-left: 3px solid var(--success-color);
}

.node-card {
  border-left: 3px solid var(--primary-color);
}

.proxy-name-container {
  font-weight: 500;
  flex: 1;
  overflow: hidden;
  color: var(--n-text-color-1);
}

.delay-tag,
.type-tag {
  font-weight: 500;
  transition: all 0.3s ease;
}

.node-type {
  margin-top: -8px;
}

.proxy-button {
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.25s ease;
}

.proxy-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

:deep(.dark) .proxy-button:hover:not(:disabled) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
}

.nodes-filter {
  margin-bottom: 20px;
}

.search-input {
  max-width: 320px;
  border-radius: 20px;
}

.empty-container {
  margin: 60px 0;
  opacity: 0.8;
}

:deep(.proxy-tabs .n-tabs-tab) {
  padding: 8px 16px;
  font-weight: 500;
  transition: all 0.3s ease;
}

:deep(.proxy-tabs .n-tabs-tab.n-tabs-tab--active) {
  font-weight: 600;
}

:deep(.proxy-tabs .n-tabs-tab-wrapper) {
  padding: 4px;
}

:deep(.n-tabs .n-tab-pane) {
  padding: 16px 0;
}

:deep(.n-card.proxy-node-card) {
  background-color: var(--card-color);
}

:deep(.n-card.proxy-node-card:hover) {
  background-color: var(--card-color-hover);
}

.batch-test-progress {
  margin-top: 16px;
  margin-bottom: 16px;
  width: 100%;
}

@keyframes slide-up {
  0% {
    opacity: 0;
    transform: translateY(20px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
