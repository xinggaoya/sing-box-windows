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
              {{ t('proxy.title') }}
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
                {{ t('proxy.modeSwitchTip') }}
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
              {{ t('proxy.refreshList') }}
            </n-tooltip>
          </div>
        </div>
      </template>
    </n-card>

    <!-- 代理模式切换对话框 -->
    <n-modal
      v-model:show="showModeChangeModal"
      preset="dialog"
      :title="`${t('proxy.switchTo')}${targetProxyMode ? getProxyModeText(targetProxyMode) : ''}`"
    >
      <template #header>
        <div class="modal-header">
          <n-icon size="22" class="modal-icon">
            <information-circle-outline />
          </n-icon>
          <span
            >{{ t('proxy.switchTo')
            }}{{ targetProxyMode ? getProxyModeText(targetProxyMode) : '' }}</span
          >
        </div>
      </template>
      <div class="modal-content">{{ t('proxy.switchModeConfirm') }}</div>
      <template #action>
        <div class="modal-footer">
          <n-space justify="end">
            <n-button @click="showModeChangeModal = false">{{ t('common.cancel') }}</n-button>
            <n-button type="primary" :loading="isChangingMode" @click="confirmProxyModeChange">
              {{ t('proxy.confirmSwitch') }}
            </n-button>
          </n-space>
        </div>
      </template>
    </n-modal>

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
                <n-space align="center" :size="12">
                  <n-tag :bordered="false" type="success" size="medium" class="proxy-tag">
                    {{ t('proxy.currentNode') }}: {{ group.now }}
                  </n-tag>
                  <n-tag :bordered="false" type="info" size="medium" class="proxy-tag">
                    {{ group.all.length }} {{ t('proxy.nodeCount') }}
                  </n-tag>
                  <n-tag :bordered="false" type="warning" size="medium" class="proxy-tag">
                    {{ group.type }}
                  </n-tag>
                  <n-button
                    @click="testNodeDelay(group.name)"
                    :loading="testingGroup === group.name"
                    secondary
                    size="small"
                    type="info"
                    ghost
                    class="proxy-button"
                  >
                    <template #icon>
                      <n-icon><speedometer-outline /></n-icon>
                    </template>
                    {{ t('proxy.speedTest') }}
                  </n-button>
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
                            getNodeDelay(proxy) === 0
                              ? t('proxy.notTested')
                              : getNodeDelay(proxy) + 'ms'
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
                          {{ group.now === proxy ? t('proxy.inUse') : t('proxy.switch') }}
                        </n-button>
                      </n-flex>
                    </n-space>
                  </n-card>
                </n-grid-item>
              </n-grid>
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

// 状态定义
const message = useMessage()
const isLoading = ref(false)
const { width } = useWindowSize()
const { t } = useI18n()

// 代理数据
const rawProxies = ref<Record<string, ProxyData>>({})
const proxyGroups = ref<ProxyData[]>([])
const testingNodes = reactive<Record<string, boolean>>({})
const currentProxyMode = ref('rule') // 默认为规则模式

// 代理模式切换相关
const isChangingMode = ref(false)
const showModeChangeModal = ref(false)
const targetProxyMode = ref('')

// 注册事件监听器
let unlistenTestProgress: (() => void) | null = null
let unlistenTestResult: (() => void) | null = null
let unlistenTestComplete: (() => void) | null = null

// 代理模式选项
const proxyModeOptions = [
  {
    label: t('proxy.mode.global'),
    key: 'global',
    icon: renderIcon(GlobeOutline),
  },
  {
    label: t('proxy.mode.rule'),
    key: 'rule',
    icon: renderIcon(LayersOutline),
  },
]

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

// 在 script setup 部分添加新的状态和方法
const testingGroup = ref('')
const testResults = reactive<Record<string, number>>({})

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
 * 获取节点的延迟
 * @param name 节点名称
 * @returns 延迟值（毫秒）
 */
const getNodeDelay = (name: string): number => {
  return testResults[name] || 0
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
    global: t('proxy.mode.global'),
    rule: t('proxy.mode.rule'),
    tun: t('proxy.mode.tun'),
  }
  return modeMap[mode] || t('proxy.mode.unknown')
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
 * @param group 代理组名称
 */
const testNodeDelay = async (group: string) => {
  if (testingGroup.value === group) return

  testingGroup.value = group
  try {
    await tauriApi.proxy.testGroupDelay(group)
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

/**
 * 获取当前代理模式
 */
const getCurrentProxyMode = async () => {
  try {
    // 调用后端API获取当前代理模式
    const mode = await tauriApi.proxy.getCurrentProxyMode()
    currentProxyMode.value = mode
    console.log(t('proxy.currentMode'), mode)
  } catch (error) {
    console.error(t('proxy.getModeError'), error)
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
    await tauriApi.proxy.toggleProxyMode(targetProxyMode.value)
    await tauriApi.kernel.restartKernel()
    currentProxyMode.value = targetProxyMode.value
    message.success(t('proxy.modeChangeSuccess', { mode: getProxyModeText(targetProxyMode.value) }))
    // 重新加载数据
    await init()
  } catch (error) {
    console.error(t('proxy.modeChangeFailed'), error)
    message.error(`${t('proxy.modeChangeError')}: ${error}`)
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

.proxy-name-container {
  font-weight: 500;
  flex: 1;
  overflow: hidden;
  color: var(--n-text-color-1);
}

.delay-tag {
  font-weight: 500;
  transition: all 0.3s ease;
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

:deep(.n-tabs .n-tab-pane) {
  padding: 16px 0;
}

:deep(.n-card.proxy-node-card) {
  background-color: var(--card-color);
}

:deep(.n-card.proxy-node-card:hover) {
  background-color: var(--card-color-hover);
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
