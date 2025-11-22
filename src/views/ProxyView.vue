<template>
  <div class="page-container">
    <PageHeader :title="t('proxy.title')" :subtitle="t('proxy.subtitle')">
      <template #actions>
        <n-button
          @click="init"
          :loading="isLoading"
          type="primary"
          secondary
          round
        >
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          {{ t('common.refresh') }}
        </n-button>
      </template>
    </PageHeader>

    <!-- Stats Grid -->
    <div class="stats-grid">
      <StatusCard
        v-for="stat in proxyStats"
        :key="stat.label"
        :label="stat.label"
        :value="stat.value"
        :type="stat.type"
      >
        <template #icon>
          <n-icon><component :is="stat.icon" /></n-icon>
        </template>
      </StatusCard>
    </div>

    <!-- Content -->
    <div class="content-section">
      <n-spin :show="isLoading">
        <template #description>
          <span class="loading-text">{{ t('proxy.loadingInfo') }}</span>
        </template>

        <!-- Empty State -->
        <div v-if="proxyGroups.length === 0 && !isLoading" class="empty-state">
          <div class="empty-icon">
            <n-icon size="48"><GlobeOutline /></n-icon>
          </div>
          <h3 class="empty-title">{{ t('proxy.noProxyGroups') }}</h3>
          <p class="empty-desc">{{ t('proxy.checkConfigOrRefresh') }}</p>
          <n-button @click="init" type="primary">
            {{ t('common.refresh') }}
          </n-button>
        </div>

        <!-- Proxy Groups -->
        <div v-else class="proxy-groups">
          <div
            v-for="group in [...proxyGroups].reverse()"
            :key="group.name"
            class="group-card"
          >
            <!-- Group Header -->
            <div class="group-header" @click="toggleGroup(group)">
              <div class="group-info">
                <div class="group-title-row">
                  <h3 class="group-name">{{ group.name }}</h3>
                  <n-tag size="small" round :bordered="false" type="primary" secondary>
                    {{ group.type }}
                  </n-tag>
                  <n-tag size="small" round :bordered="false">
                    {{ group.all.length }} {{ t('proxy.nodes') }}
                  </n-tag>
                </div>
                <div class="group-status">
                  <span class="label">{{ t('proxy.currentLabel') }}:</span>
                  <span class="value">{{ group.now }}</span>
                </div>
              </div>

              <div class="group-actions">
                <n-button
                  @click.stop="testNodeDelay(group.name)"
                  :loading="testingGroup === group.name"
                  size="small"
                  secondary
                  round
                >
                  <template #icon>
                    <n-icon><SpeedometerOutline /></n-icon>
                  </template>
                  {{ t('proxy.testNode') }}
                </n-button>
                <div class="expand-btn" :class="{ expanded: expandedGroups.includes(group.name) }">
                  <n-icon><ChevronDownOutline /></n-icon>
                </div>
              </div>
            </div>

            <!-- Nodes Grid -->
            <transition name="expand">
              <div v-if="expandedGroups.includes(group.name)" class="nodes-area">
                <div class="nodes-grid">
                  <div
                    v-for="(proxy, i) in getVisibleNodes(group)"
                    :key="`${group.name}-${proxy}-${i}`"
                    class="node-item"
                    :class="{
                      active: group.now === proxy,
                      testing: testingNodes[proxy],
                      error: nodeErrors[proxy]
                    }"
                    @click="changeProxy(group.name, proxy)"
                  >
                    <div class="node-status-icon">
                      <n-icon v-if="group.now === proxy"><CheckmarkCircleOutline /></n-icon>
                      <n-icon v-else-if="testingNodes[proxy]" class="spin"><RefreshOutline /></n-icon>
                      <div v-else class="dot"></div>
                    </div>
                    
                    <div class="node-details">
                      <div class="node-name" :title="proxy">{{ proxy }}</div>
                      <div class="node-meta" @click.stop="testSingleNode(proxy)">
                        {{ getNodeStatusText(proxy) }}
                      </div>
                    </div>
                  </div>
                </div>
                <div class="nodes-footer">
                  <div class="nodes-count">
                    {{ t('proxy.loadedCount', { loaded: getVisibleNodes(group).length, total: group.all.length }) }}
                  </div>
                  <n-button
                    v-if="hasMoreNodes(group)"
                    size="small"
                    tertiary
                    @click.stop="loadMoreNodes(group)"
                  >
                    {{ t('proxy.loadMoreNodes') }}
                  </n-button>
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
import { tauriApi } from '@/services/tauri'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores'
import PageHeader from '@/components/common/PageHeader.vue'
import StatusCard from '@/components/common/StatusCard.vue'

// Interfaces
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

interface TestGroupResult {
  [proxyName: string]: number
}

interface TestNodeResult {
  proxy: string
  delay: number
}

defineOptions({
  name: 'ProxyView'
})

const message = useMessage()
const isLoading = ref(false)
const { t } = useI18n()
const appStore = useAppStore()

// Data
const rawProxies = ref<Record<string, ProxyData>>({})
const proxyGroups = ref<ProxyData[]>([])
const testingNodes = reactive<Record<string, boolean>>({})
const expandedGroups = ref<string[]>([])
const testingGroup = ref('')
const testResults = reactive<Record<string, number>>({})
const nodeErrors = reactive<Record<string, string>>({})
const groupNodeState = reactive<Record<string, { list: string[]; loaded: number }>>({})
const NODE_BATCH_SIZE = 60

// Listeners
let unlistenTestProgress: (() => void) | null = null
let unlistenTestResult: (() => void) | null = null
let unlistenTestComplete: (() => void) | null = null
let unlistenNodeResult: (() => void) | null = null

// Computed
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
      type: 'primary' as const,
    },
    {
      label: t('proxy.dashboard.nodeTotal'),
      value: totalNodes,
      icon: GlobeOutline,
      type: 'success' as const,
    },
    {
      label: t('proxy.dashboard.expanded'),
      value: expanded,
      icon: ChevronDownOutline,
      type: 'warning' as const,
    },
    {
      label: t('proxy.dashboard.testing'),
      value: testingCount,
      icon: SpeedometerOutline,
      type: 'default' as const,
    },
  ]
})

// Methods
// 懒加载节点，按批次渲染减少展开大组时的卡顿
const ensureGroupState = (group: ProxyData) => {
  if (!groupNodeState[group.name]) {
    groupNodeState[group.name] = { list: [], loaded: 0 }
  }
  return groupNodeState[group.name]
}

const loadNextBatch = (group: ProxyData) => {
  const state = ensureGroupState(group)
  if (state.loaded >= (group.all?.length ?? 0)) return
  const next = group.all.slice(state.loaded, state.loaded + NODE_BATCH_SIZE)
  state.list.push(...next)
  state.loaded += next.length
}

const getVisibleNodes = (group: ProxyData) => {
  const state = groupNodeState[group.name]
  return state ? state.list : []
}

const hasMoreNodes = (group: ProxyData) => {
  const state = groupNodeState[group.name]
  return (state?.loaded ?? 0) < (group.all?.length ?? 0)
}

const loadMoreNodes = (group: ProxyData) => {
  loadNextBatch(group)
}

const resetGroupNodeState = (groups: ProxyData[]) => {
  Object.keys(groupNodeState).forEach((key) => delete groupNodeState[key])
  groups.forEach((group) => {
    groupNodeState[group.name] = { list: [], loaded: 0 }
  })
}

const collapseGroup = (groupName: string) => {
  const index = expandedGroups.value.indexOf(groupName)
  if (index > -1) expandedGroups.value.splice(index, 1)
  delete groupNodeState[groupName]
}

const toggleGroup = (group: ProxyData) => {
  const isExpanded = expandedGroups.value.includes(group.name)
  if (isExpanded) {
    collapseGroup(group.name)
    return
  }
  // 仅保持一个展开的组，避免同时渲染多个组造成卡顿
  expandedGroups.value.forEach((name) => collapseGroup(name))
  expandedGroups.value = [group.name]
  loadNextBatch(group)
}

const getNodeStatusText = (name: string): string => {
  if (testingNodes[name]) return t('proxy.testing')
  if (nodeErrors[name]) return t('proxy.timeout')
  const delay = testResults[name] || 0
  return delay === 0 ? '--' : `${delay}ms`
}

const init = async () => {
  isLoading.value = true
  try {
    const data = await tauriApi.proxy.getProxies()
    rawProxies.value = data.proxies
    const groups: ProxyData[] = []
    Object.entries(data.proxies).forEach(([key, item]) => {
      if (key === 'GLOBAL' || key === 'direct') return
      if (item.type === 'Selector') {
        groups.push(item)
      }
    })
    proxyGroups.value = groups
    expandedGroups.value = []
    resetGroupNodeState(groups)
    if (groups.length > 0) {
      message.success(t('proxy.loadSuccess'))
    }
  } catch (error) {
    message.error(t('proxy.loadFailedCheck'))
  } finally {
    isLoading.value = false
  }
}

const testSingleNode = async (proxy: string) => {
  if (testingNodes[proxy]) return
  testingNodes[proxy] = true
  try {
    delete nodeErrors[proxy]
    await tauriApi.proxy.testNodeDelay(proxy)
  } catch (error) {
    testingNodes[proxy] = false
    nodeErrors[proxy] = t('proxy.timeout')
  }
}

const testNodeDelay = async (group: string) => {
  if (testingGroup.value === group) return
  testingGroup.value = group
  try {
    await tauriApi.proxy.testGroupDelay(group)
  } catch (error) {
    message.error(`${t('proxy.testErrorMessage')}: ${group}`)
    testingGroup.value = ''
  }
}

const changeProxy = async (group: string, proxy: string) => {
  try {
    await tauriApi.proxy.changeProxy(group, proxy)
    message.success(t('proxy.switchSuccess', { group, proxy }))
    await init()
    await testNodeDelay(group)
  } catch (error) {
    message.error(t('proxy.switchErrorMessage'))
  }
}

const setupEventListeners = async () => {
  unlistenTestProgress = await listen('test-nodes-progress', (event) => {
    // Optional: Update progress
  })

  unlistenTestResult = await listen('proxy-group-delay-result', (event) => {
    const data = event.payload as TestGroupResult
    if (data && typeof data === 'object') {
      Object.entries(data).forEach(([proxyName, delay]) => {
        if (typeof delay === 'number') testResults[proxyName] = delay
      })
      message.success(t('proxy.groupTestComplete'))
    }
    testingGroup.value = ''
  })

  unlistenTestComplete = await listen('test-nodes-complete', () => {
    message.success(t('proxy.batchTestComplete'))
  })

  unlistenNodeResult = await listen('proxy-delay-result', (event) => {
    const data = event.payload as TestNodeResult
    if (data && data.proxy) {
      const { proxy, delay } = data
      testingNodes[proxy] = false
      if (delay !== undefined && delay > 0) {
        testResults[proxy] = delay
        delete nodeErrors[proxy]
      } else {
        nodeErrors[proxy] = t('proxy.timeout')
      }
    }
  })
}

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
</script>

<style scoped>
.page-container {
  padding: 24px 32px;
  max-width: 1400px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.content-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  color: var(--text-secondary);
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 8px;
  color: var(--text-primary);
}

.empty-desc {
  margin: 0 0 24px;
}

/* Proxy Groups */
.proxy-groups {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.group-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  overflow: hidden;
  transition: all 0.2s ease;
}

.group-card:hover {
  border-color: var(--border-hover);
  box-shadow: var(--panel-shadow);
}

.group-header {
  padding: 16px 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
  background: rgba(255, 255, 255, 0.02);
}

.group-header:hover {
  background: var(--bg-tertiary);
}

.group-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-title-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.group-name {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.group-status {
  font-size: 13px;
  display: flex;
  gap: 6px;
}

.group-status .label {
  color: var(--text-tertiary);
}

.group-status .value {
  color: var(--success-color, #10b981);
  font-weight: 500;
}

.group-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.expand-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  color: var(--text-secondary);
  transition: all 0.2s ease;
}

.expand-btn.expanded {
  transform: rotate(180deg);
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

/* Nodes Grid */
.nodes-area {
  padding: 20px 24px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-primary);
}

.nodes-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.nodes-footer {
  margin-top: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 12px;
  color: var(--text-tertiary);
}

.nodes-count {
  flex: 1;
}

.node-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
  background: var(--panel-bg);
  cursor: pointer;
  transition: all 0.2s ease;
}

.node-item:hover {
  border-color: var(--primary-color);
  transform: translateY(-2px);
}

.node-item.active {
  background: rgba(16, 185, 129, 0.1);
  border-color: #10b981;
}

.node-item.error {
  border-color: #ef4444;
  background: rgba(239, 68, 68, 0.05);
}

.node-status-icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.node-item.active .node-status-icon {
  color: #10b981;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--border-color);
}

.node-item:hover .dot {
  background: var(--primary-color);
}

.node-details {
  flex: 1;
  min-width: 0;
}

.node-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-meta {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s ease;
  max-height: 1000px;
  opacity: 1;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
  padding-top: 0;
  padding-bottom: 0;
}
</style>
