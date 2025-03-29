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
              {{ t('proxy.settings') }}
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
                      <layers-outline v-else-if="currentProxyMode === 'rule'" />
                      <hardware-chip-outline v-else-if="currentProxyMode === 'tun'" />
                    </n-icon>
                    {{ getProxyModeText(currentProxyMode) }}
                    <n-icon size="16" class="dropdown-icon">
                      <chevron-down-outline />
                    </n-icon>
                  </n-tag>
                </template>
                {{ t('proxy.click_to_change_mode') }}
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
              {{ t('proxy.refresh_list') }}
            </n-tooltip>
          </div>
        </div>
      </template>
    </n-card>

    <!-- 代理模式切换对话框 -->
    <n-modal
      v-model:show="showModeChangeModal"
      preset="dialog"
      :title="t('proxy.modal_title', { mode: targetProxyMode ? getProxyModeText(targetProxyMode) : '' })"
    >
      <template #header>
        <div class="modal-header">
          <n-icon size="22" class="modal-icon">
            <information-circle-outline />
          </n-icon>
          <span>{{ t('proxy.modal_header', { mode: targetProxyMode ? getProxyModeText(targetProxyMode) : '' }) }}</span>
        </div>
      </template>
      <div class="modal-content">
        {{ t('proxy.modal_content') }}
      </div>
      <template #action>
        <div class="modal-footer">
          <n-space justify="end">
            <n-button @click="showModeChangeModal = false">{{ t('proxy.cancel') }}</n-button>
            <n-button type="primary" :loading="isChangingMode" @click="confirmProxyModeChange">
              {{ t('proxy.confirm_switch') }}
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
                    {{ t('proxy.current_node', { node: group.now }) }}
                  </n-tag>
                  <n-tag :bordered="false" type="info" size="medium" class="proxy-tag">
                    {{ group.all.length }} {{ t('proxy.nodes') }}
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
                    {{ t('proxy.test_speed') }}
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
                          {{ getNodeDelay(proxy) === 0 ? t('proxy.not_tested') : getNodeDelay(proxy) + 'ms' }}
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
                          {{ group.now === proxy ? t('proxy.in_use') : t('proxy.switch') }}
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
const { t } = useI18n()
const message = useMessage()
const isLoading = ref(false)
const { width } = useWindowSize()

// 代理数据
const rawProxies = ref<Record<string, any>>({})
const proxyGroups = ref<any[]>([])
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
    label: t('proxy.mode_options.global'),
    key: 'global',
    icon: renderIcon(GlobeOutline),
  },
  {
    label: t('proxy.mode_options.rule'),
    key: 'rule',
    icon: renderIcon(LayersOutline),
  },
]

// Динамическая отрисовка иконок
function renderIcon(icon: Component) {
  return () => h('div', { class: 'dropdown-option-icon' }, h(icon))
}

// Состояние вкладок
const activeGroupTab = ref('')

// Подсчет количества столбцов в сетке в зависимости от ширины окна
const gridCols = computed(() => {
  if (width.value < 640) return 1
  if (width.value < 960) return 2
  if (width.value < 1280) return 3
  return 4
})

// Новые состояния и методы для тестирования
const testingGroup = ref('')
const testResults = reactive<Record<string, number>>({})

onMounted(() => {
  init()
  getCurrentProxyMode()
  setupEventListeners()
})

onUnmounted(() => {
  if (unlistenTestProgress) unlistenTestProgress()
  if (unlistenTestResult) unlistenTestResult()
  if (unlistenTestComplete) unlistenTestComplete()
})

const setupEventListeners = async () => {
  unlistenTestProgress = await listen('test-nodes-progress', (event) => {
    const data = event.payload as { current: number; total: number; node: string; status: string }
    console.log('测试进度:', data)
  })

  unlistenTestResult = await listen('test-group-result', (event) => {
    const data = event.payload as { group: string; results: Record<string, number>; success: boolean; error?: string }
    if (data.success) {
      Object.assign(testResults, data.results)
      message.success(t('proxy.test_group_success'))
    } else {
      message.error(t('proxy.test_group_fail', { error: data.error }))
    }
    testingGroup.value = ''
  })

  unlistenTestComplete = await listen('test-nodes-complete', () => {
    message.success(t('proxy.test_batch_success'))
  })
}

/**
 * Инициализация и получение информации о прокси
 */
const init = async () => {
  isLoading.value = true
  try {
    const data = await tauriApi.proxy.getProxies()
    rawProxies.value = data.proxies

    const groups: any[] = []
    Object.entries(data.proxies).forEach(([key, item]) => {
      if (key === 'GLOBAL' || key === 'direct') return
      if (item.type === 'Selector' || item.type === 'URLTest') {
        groups.push(item)
      }
    })
    proxyGroups.value = groups

    if (groups.length > 0) {
      activeGroupTab.value = groups.length >= 3 ? groups[groups.length - 3].name : groups[0].name
      message.success(t('proxy.load_success'))
    }
  } catch (error) {
    console.error('获取代理列表失败', error)
    message.error(t('proxy.load_fail'))
  } finally {
    isLoading.value = false
  }
}

/**
 * Получение задержки узла
 */
const getNodeDelay = (name: string): number => {
  return testResults[name] || 0
}

/**
 * Определение типа задержки для отображения цвета
 */
const getNodeDelayType = (delay: number): string => {
  if (delay === 0) return 'default'
  if (delay < 100) return 'success'
  if (delay < 200) return 'info'
  if (delay < 300) return 'warning'
  return 'error'
}

/**
 * Получение текста для режима прокси с использованием i18n
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
 * Тестирование задержки группы узлов
 */
const testNodeDelay = async (group: string) => {
  if (testingGroup.value === group) return
  testingGroup.value = group
  try {
    await tauriApi.proxy.testGroupDelay(group)
  } catch (error) {
    console.error('测速失败', error)
    message.error(t('proxy.test_failed'))
    testingGroup.value = ''
  }
}

/**
 * Смена прокси
 */
const changeProxy = async (group: string, proxy: string) => {
  try {
    await tauriApi.proxy.changeProxy(group, proxy)
    message.success(t('proxy.changed', { group, proxy }))
    await init()
    await testNodeDelay(group)
  } catch (error) {
    console.error('切换失败', error)
    message.error(t('proxy.switch_error'))
  }
}

/**
 * Получение текущего режима прокси
 */
const getCurrentProxyMode = async () => {
  try {
    const mode = await tauriApi.proxy.getCurrentProxyMode()
    currentProxyMode.value = mode
    console.log('当前代理模式:', mode)
  } catch (error) {
    console.error('获取代理模式失败', error)
    currentProxyMode.value = 'rule'
  }
}

/**
 * Обработка изменения режима прокси
 */
const handleProxyModeChange = (key: string) => {
  if (key === currentProxyMode.value) return
  targetProxyMode.value = key
  showModeChangeModal.value = true
}

/**
 * Подтверждение смены режима прокси
 */
 const confirmProxyModeChange = async () => {
  if (!targetProxyMode.value) return

  isChangingMode.value = true
  try {
    await tauriApi.proxy.toggleProxyMode(targetProxyMode.value)
    await tauriApi.kernel.restartKernel()
    currentProxyMode.value = targetProxyMode.value
    message.success(`已切换到${getProxyModeText(targetProxyMode.value)}并重启内核`)
    // 重新加载数据
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
/* Стили оставлены без изменений */
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
