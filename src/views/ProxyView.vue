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

  
  <!-- Content -->
  <div class="content-section">
    <div class="filter-bar">
      <n-input
        v-model:value="nodeSearch"
        :placeholder="t('proxy.searchNode')"
        clearable
        size="small"
        round
        class="filter-input"
      >
        <template #prefix>
          <n-icon><SearchOutline /></n-icon>
        </template>
      </n-input>
      <n-button
        size="small"
        tertiary
        :type="showFavoritesOnly ? 'primary' : 'default'"
        @click="toggleFavoritesFilter"
      >
        <template #icon>
          <n-icon><component :is="showFavoritesOnly ? Star : StarOutline" /></n-icon>
        </template>
        {{ t('proxy.onlyFavorites') }}
      </n-button>
      <n-button
        size="small"
        secondary
        :loading="batchTesting"
        @click="batchTestAllNodes"
      >
        <template #icon>
          <n-icon><SpeedometerOutline /></n-icon>
        </template>
        {{ t('proxy.batchTest') }}
      </n-button>
    </div>

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
                  @click.stop="autoSelectBest(group)"
                  size="small"
                  tertiary
                  round
                >
                  <template #icon>
                    <n-icon><CheckmarkCircleOutline /></n-icon>
                  </template>
                  {{ t('proxy.autoSelect') }}
                </n-button>
                <n-button
                  @click.stop="testGroupNodes(group)"
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
                      <div class="node-meta">
                        <span class="status-text" @click.stop="testSingleNode(proxy)">
                          {{ getNodeStatusText(proxy) }}
                        </span>
                        <n-button
                          text
                          size="tiny"
                          :loading="testingNodes[proxy]"
                          @click.stop="testSingleNode(proxy)"
                        >
                          <template #icon>
                            <n-icon size="14"><SpeedometerOutline /></n-icon>
                          </template>
                          {{ t('proxy.testNode') }}
                        </n-button>
                      </div>
                    </div>
                    <div class="node-actions">
                      <n-button text size="tiny" @click.stop="toggleFavorite(proxy)">
                        <n-icon :class="{ favorite: isFavorite(proxy) }">
                          <component :is="isFavorite(proxy) ? Star : StarOutline" />
                        </n-icon>
                      </n-button>
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
import { onMounted, ref, reactive, watch } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  CheckmarkCircleOutline,
  SpeedometerOutline,
  GlobeOutline,
  ChevronDownOutline,
  SearchOutline,
  StarOutline,
  Star,
} from '@vicons/ionicons5'
import { proxyService } from '@/services/proxy-service'
import { useI18n } from 'vue-i18n'
import PageHeader from '@/components/common/PageHeader.vue'

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

defineOptions({
  name: 'ProxyView'
})

const message = useMessage()
const isLoading = ref(false)
const { t } = useI18n()

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
const nodeSearch = ref('')
const showFavoritesOnly = ref(false)
const favoriteNodes = ref<string[]>([])
const batchTesting = ref(false)

watch([nodeSearch, showFavoritesOnly], () => {
  resetGroupNodeState(proxyGroups.value)
})

// Methods
const loadFavorites = () => {
  try {
    const saved = localStorage.getItem('sbw_favorite_nodes')
    if (saved) {
      favoriteNodes.value = JSON.parse(saved)
    }
  } catch {
    favoriteNodes.value = []
  }
}

const persistFavorites = () => {
  localStorage.setItem('sbw_favorite_nodes', JSON.stringify(favoriteNodes.value))
}

const toggleFavorite = (node: string) => {
  const idx = favoriteNodes.value.indexOf(node)
  if (idx === -1) {
    favoriteNodes.value.push(node)
  } else {
    favoriteNodes.value.splice(idx, 1)
  }
  persistFavorites()
  if (showFavoritesOnly.value) {
    resetGroupNodeState(proxyGroups.value)
  }
}

const isFavorite = (node: string) => favoriteNodes.value.includes(node)

const toggleFavoritesFilter = () => {
  showFavoritesOnly.value = !showFavoritesOnly.value
  resetGroupNodeState(proxyGroups.value)
}

const getFilteredNodesList = (group: ProxyData) => {
  const search = nodeSearch.value.trim().toLowerCase()
  return (group.all || []).filter((node) => {
    const matchSearch = !search || node.toLowerCase().includes(search)
    const matchFavorite = !showFavoritesOnly.value || favoriteNodes.value.includes(node)
    return matchSearch && matchFavorite
  })
}

const resetGroupNodeState = (groups: ProxyData[]) => {
  groups.forEach(group => {
    const prev = groupNodeState[group.name]
    const list = getFilteredNodesList(group)
    const loaded = prev ? Math.min(prev.loaded, list.length) : Math.min(NODE_BATCH_SIZE, list.length)
    groupNodeState[group.name] = {
      list,
      loaded
    }
  })
}

const getVisibleNodes = (group: ProxyData) => {
  const state = groupNodeState[group.name]
  const list = state ? state.list : getFilteredNodesList(group)
  return list.slice(0, state?.loaded ?? NODE_BATCH_SIZE)
}

const hasMoreNodes = (group: ProxyData) => {
  const state = groupNodeState[group.name]
  const list = state ? state.list : getFilteredNodesList(group)
  if (!state) return list.length > NODE_BATCH_SIZE
  return state.loaded < list.length
}

const loadMoreNodes = (group: ProxyData) => {
  const state = groupNodeState[group.name]
  if (state) {
    state.loaded += NODE_BATCH_SIZE
  }
}

const toggleGroup = (group: ProxyData) => {
  const index = expandedGroups.value.indexOf(group.name)
  if (index === -1) {
    expandedGroups.value.push(group.name)
  } else {
    expandedGroups.value.splice(index, 1)
  }
}

const getNodeStatusText = (proxy: string) => {
  if (testingNodes[proxy]) return t('proxy.testing')
  if (nodeErrors[proxy]) return nodeErrors[proxy]
  const delay = testResults[proxy]
  if (delay !== undefined) return `${delay} ms`
  return t('proxy.clickToTest')
}

const init = async (options: { preserveExpanded?: boolean } = {}) => {
  const { preserveExpanded = true } = options
  const previousExpanded = preserveExpanded ? new Set(expandedGroups.value) : new Set<string>()
  isLoading.value = true
  try {
    const data = await proxyService.getProxies()
    rawProxies.value = data.proxies
    const groups: ProxyData[] = []
    Object.entries(data.proxies).forEach(([key, item]) => {
      if (key === 'GLOBAL' || key === 'direct') return
      if (item.type === 'Selector') {
        groups.push(item)
      }
    })
    proxyGroups.value = groups
    expandedGroups.value = preserveExpanded
      ? groups.map(item => item.name).filter(name => previousExpanded.has(name))
      : []
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

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

const normalizeDelayError = (raw?: string | null) => {
  if (!raw) return t('proxy.testFailed')
  const text = raw.toLowerCase()

  // 列表里尽量只展示简短状态，详细原因交给 toast 或日志。
  if (text.includes('timeout') || text.includes('timed out')) return t('proxy.timeout')
  if (text.includes('delay=0')) return t('proxy.testFailed')
  if (text.includes('http')) return t('proxy.testFailed')

  return t('proxy.testFailed')
}

// 统一的“多节点测速”执行器：单测/组测/批测全部复用，避免逻辑分叉导致体验不一致。
const runDelayTests = async (
  nodes: string[],
  options: { concurrency?: number; gapMs?: number } = {},
) => {
  const list = Array.from(new Set(nodes)).filter(Boolean)
  if (!list.length) return

  const concurrency = Math.min(options.concurrency ?? 6, list.length)
  const gapMs = options.gapMs ?? 120
  let pointer = 0

  const runner = async () => {
    while (pointer < list.length) {
      const current = list[pointer]
      pointer += 1
      await testSingleNode(current)
      if (gapMs > 0) await sleep(gapMs)
    }
  }

  await Promise.all(Array.from({ length: concurrency }, runner))
}

const testSingleNode = async (proxy: string) => {
  if (testingNodes[proxy]) return
  testingNodes[proxy] = true

  // 说明：旧实现依赖事件回传（listen + emit），在组测速场景下会出现“部分节点无结果也无错误”。
  // 现在改为直接等待 invoke 返回结果，确保每个节点都有明确的 ok/error。
  try {
    delete nodeErrors[proxy]
    const result = await proxyService.testNodeDelay(proxy)
    if (result.ok && result.delay > 0) {
      testResults[proxy] = result.delay
      delete nodeErrors[proxy]
    } else {
      nodeErrors[proxy] = normalizeDelayError(result.error)
    }
  } catch (error) {
    nodeErrors[proxy] = t('proxy.testFailed')
  } finally {
    testingNodes[proxy] = false
  }
}

const testGroupNodes = async (group: ProxyData) => {
  if (testingGroup.value === group.name || batchTesting.value) return

  const nodes = getFilteredNodesList(group)
  if (!nodes.length) {
    message.warning(t('proxy.noProxyGroups'))
    return
  }

  testingGroup.value = group.name
  try {
    await runDelayTests(nodes, { concurrency: 6, gapMs: 120 })
    message.success(t('proxy.groupTestComplete'))
  } catch (error) {
    message.error(`${t('proxy.testErrorMessage')}: ${group.name}`)
  } finally {
    testingGroup.value = ''
  }
}

const batchTestAllNodes = async () => {
  if (batchTesting.value || testingGroup.value) return

  const nodes = new Set<string>()
  proxyGroups.value.forEach(group => (group.all || []).forEach(node => nodes.add(node)))
  if (nodes.size === 0) {
    message.warning(t('proxy.noProxyGroups'))
    return
  }

  batchTesting.value = true
  try {
    await runDelayTests(Array.from(nodes), { concurrency: 6, gapMs: 120 })
    message.success(t('proxy.batchTestComplete'))
  } catch (error) {
    message.error(t('proxy.testErrorMessage'))
  } finally {
    batchTesting.value = false
  }
}

const autoSelectBest = async (group: ProxyData) => {
  const nodes = getFilteredNodesList(group)
  if (!nodes.length) {
    message.warning(t('proxy.noProxyGroups'))
    return
  }

  let bestNode: { node: string | null; delay: number } = { node: null, delay: Number.MAX_SAFE_INTEGER }
  nodes.forEach(node => {
    // 兜底：即便用户导入了非本程序生成的配置，也避免“自动选优”选到 direct 导致全直连。
    if (node === 'direct') return
    const delay = testResults[node]
    if (delay && delay > 0 && delay < bestNode.delay) {
      bestNode = { node, delay }
    }
  })

  if (!bestNode.node) {
    await testGroupNodes(group)
    message.info(t('proxy.testNode'))
    return
  }

  await changeProxy(group.name, bestNode.node)
}

const changeProxy = async (group: string, proxy: string) => {
  if (testingNodes[proxy]) {
    message.info(t('proxy.testing'))
    return
  }
  try {
    await proxyService.changeProxy(group, proxy)
    const targetGroup = proxyGroups.value.find(item => item.name === group)
    if (targetGroup) {
      targetGroup.now = proxy
    }
    if (rawProxies.value[group]) {
      rawProxies.value[group].now = proxy
    }
    message.success(t('proxy.switchSuccess', { group, proxy }))
  } catch (error) {
    message.error(t('proxy.switchErrorMessage'))
  }
}

/* 旧的基于事件回传的测速实现已废弃（组测速可能丢结果导致 UI 无提示）。
 * 当前页面统一使用 invoke 返回结果的方式，确保每个节点都有明确的 ok/error。

const setupEventListeners = async () => {
  unlistenTestProgress = await listen('test-delay-progress', (event) => {
    // 处理进度事件
  })

  unlistenTestResult = await listen('proxy-group-delay-result', (event) => {
    const data = event.payload as TestGroupResult
    if (data) {
      Object.entries(data).forEach(([proxy, delay]) => {
        if (delay > 0) {
          testResults[proxy] = delay
          delete nodeErrors[proxy]
        } else {
          nodeErrors[proxy] = t('proxy.timeout')
        }
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
*/

onMounted(() => {
  loadFavorites()
  init()
})
</script>

<style scoped>
.page-container {
  padding: var(--layout-page-padding-y, 16px) var(--layout-page-padding-x, 24px);
  max-width: var(--layout-page-max-width, 1400px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--layout-page-gap, 16px);
}


.content-section {
  display: flex;
  flex-direction: column;
  gap: var(--layout-row-gap, 16px);
}

.filter-bar {
  display: flex;
  gap: 12px;
  align-items: center;
}

.filter-input {
  max-width: 260px;
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
  display: flex;
  align-items: center;
  gap: 6px;
}

.node-actions {
  display: flex;
  align-items: center;
}

.node-actions .favorite {
  color: #f59e0b;
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
