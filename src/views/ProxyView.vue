<template>
  <div class="page-shell proxy-page">
    <ToolbarBar>
      <template #filters>
        <n-button size="small" secondary @click="refresh">
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          {{ t('common.refresh') }}
        </n-button>
        <n-button
          size="small"
          type="primary"
          secondary
          :loading="proxyStore.batchTesting"
          @click="batchTest"
        >
          <template #icon>
            <n-icon><SpeedometerOutline /></n-icon>
          </template>
          {{ t('proxy.batchTest') }}
        </n-button>
      </template>
    </ToolbarBar>

    <div class="split-view">
      <div class="group-sidebar">
        <div class="sidebar-title">{{ labels.groupsTab }}</div>
        <div class="sidebar-list">
          <div
            v-for="group in filteredGroups"
            :key="group.name"
            class="sidebar-item"
            :class="{ active: proxyStore.selectedGroup === group.name }"
            @click="selectGroup(group.name)"
          >
            <div class="sidebar-item-main">
              <span class="sidebar-item-name">{{ group.name }}</span>
              <n-tag size="tiny" round :bordered="false">{{ group.type }}</n-tag>
            </div>
            <div class="sidebar-item-now" v-if="group.now">
              <img
                v-if="getNodeFlagUrl(group.now)"
                class="node-flag"
                :src="getNodeFlagUrl(group.now)"
                alt=""
              />
              <span>{{ getDisplayNodeName(group.now) }}</span>
            </div>
          </div>
          <EmptyState
            v-if="!filteredGroups.length"
            :title="t('proxy.noProxyGroups')"
            :icon="GlobeOutline"
            class="sidebar-empty"
          />
        </div>
      </div>

      <div class="node-panel">
        <div class="node-toolbar">
          <n-input
            v-model:value="searchQuery"
            size="small"
            :placeholder="t('proxy.searchNode')"
            clearable
          >
            <template #prefix>
              <n-icon><SearchOutline /></n-icon>
            </template>
          </n-input>
          <n-select
            v-model:value="proxyStore.ordering"
            size="small"
            :options="orderingOptions"
            style="width: 120px"
          />
          <n-button
            size="small"
            quaternary
            :type="favoritesOnly ? 'primary' : 'default'"
            @click="favoritesOnly = !favoritesOnly"
          >
            <template #icon>
              <n-icon>
                <Star v-if="favoritesOnly" />
                <StarOutline v-else />
              </n-icon>
            </template>
          </n-button>
          <n-switch v-model:value="proxyStore.hideUnavailable" size="small">
            <template #checked>{{ labels.hideUnavailable }}</template>
            <template #unchecked>{{ labels.showAll }}</template>
          </n-switch>
        </div>

        <div class="node-summary" v-if="activeGroup">
          <span class="summary-text">
            {{ labels.recommended }}
            <template v-if="proxyStore.getRecommendedNode(activeGroup)">
              <img
                v-if="getNodeFlagUrl(proxyStore.getRecommendedNode(activeGroup))"
                class="node-flag"
                :src="getNodeFlagUrl(proxyStore.getRecommendedNode(activeGroup))"
                alt=""
              />
              {{ getDisplayNodeName(proxyStore.getRecommendedNode(activeGroup)) }}
            </template>
            <template v-else>-</template>
          </span>
          <span class="summary-sep">|</span>
          <span class="summary-text">{{ activeGroup.all.length }} {{ t('proxy.nodes') }}</span>
          <span class="summary-sep">|</span>
          <span class="summary-text"
            >{{ labels.favorites }}
            {{ activeGroup.all.filter((n) => proxyStore.isFavorite(n)).length }}</span
          >
          <div class="summary-spacer"></div>
          <n-button
            size="tiny"
            secondary
            @click="testActiveGroup"
            :loading="proxyStore.groupTestingMap[activeGroup.name]"
          >
            {{ t('proxy.testNode') }}
          </n-button>
          <n-button size="tiny" tertiary @click="switchRecommended(activeGroup)">
            {{ t('proxy.autoSelect') }}
          </n-button>
        </div>

        <div v-if="activeGroupNodes.length" class="node-grid">
          <div
            v-for="node in activeGroupNodes"
            :key="`${activeGroup!.name}-${node}`"
            class="node-card"
            :class="{ active: activeGroup?.now === node }"
            @click="changeProxy(activeGroup!.name, node)"
          >
            <div class="node-card-top">
              <span class="node-card-name">
                <img
                  v-if="getNodeFlagUrl(node)"
                  class="node-flag"
                  :src="getNodeFlagUrl(node)"
                  alt=""
                />
                <span>{{ getDisplayNodeName(node) }}</span>
              </span>
              <n-button
                text
                size="tiny"
                class="node-fav-btn"
                @click.stop="proxyStore.toggleFavorite(node)"
              >
                <n-icon size="14">
                  <Star v-if="proxyStore.isFavorite(node)" style="color: #f59e0b" />
                  <StarOutline v-else />
                </n-icon>
              </n-button>
            </div>
            <div class="node-card-bottom">
              <n-tag size="small" round :bordered="false" :type="latencyTagType(node)">
                {{ formatLatency(node) }}
              </n-tag>
            </div>
          </div>
        </div>

        <EmptyState
          v-else-if="activeGroup"
          :title="t('proxy.noProxyGroups')"
          :icon="GlobeOutline"
          class="node-empty"
        />

        <EmptyState
          v-else
          :title="t('proxy.noProxyGroups')"
          :icon="GlobeOutline"
          class="node-empty"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useMessage } from 'naive-ui'
import {
  GlobeOutline,
  RefreshOutline,
  SearchOutline,
  SpeedometerOutline,
  Star,
  StarOutline,
} from '@vicons/ionicons5'
import { useProxyStore, type ProxyGroup } from '@/stores/kernel/ProxyStore'
import {
  ALL_COUNTRY_CODES,
  COUNTRY_CODE_ALIASES,
  COUNTRY_DISPLAY_LOCALES,
  COUNTRY_NAME_PATTERNS,
  INFORMATIONAL_NODE_PATTERNS,
  PROXY_PREFIX_COUNTRY_CODES,
} from '@/constants/countries'
import { useI18n } from 'vue-i18n'
import ToolbarBar from '@/components/common/ToolbarBar.vue'
import EmptyState from '@/components/common/EmptyState.vue'

defineOptions({
  name: 'ProxyView',
})

const { t } = useI18n()
const message = useMessage()
const proxyStore = useProxyStore()
const searchQuery = ref('')
const favoritesOnly = ref(false)

const flagIconUrls = import.meta.glob('../../node_modules/flag-icons/flags/4x3/*.svg', {
  eager: true,
  query: '?url',
  import: 'default',
}) as Record<string, string>

const getFlagIconCountryCode = (code: string) => {
  if (code === 'EU') return 'eu'
  const normalizedCode = COUNTRY_CODE_ALIASES[code] || code
  if (!/^[A-Z]{2}$/.test(normalizedCode)) return ''
  return normalizedCode.toLowerCase()
}

const stripLeadingFlagEmoji = (nodeName: string) =>
  nodeName.replace(/^[\u{1f1e6}-\u{1f1ff}]{2}\s*/u, '').trim()

const isInformationalNodeName = (nodeName: string) =>
  INFORMATIONAL_NODE_PATTERNS.some((pattern) => pattern.test(stripLeadingFlagEmoji(nodeName)))

const normalizeCountryLabel = (value: string) =>
  value
    .toLowerCase()
    .replace(/[\s·・,，.。()（）[\]【】'’"“”_-]/g, '')
    .trim()

const getLocalizedCountryNameEntries = () => {
  const entries: Array<{ code: string; label: string }> = []
  const seen = new Set<string>()

  COUNTRY_DISPLAY_LOCALES.forEach((displayLocale) => {
    const displayNames = new Intl.DisplayNames([displayLocale], { type: 'region' })
    ALL_COUNTRY_CODES.forEach((countryCode) => {
      let label: string | undefined
      try {
        label = displayNames.of(countryCode)
      } catch {
        return
      }
      if (!label || label === countryCode) return
      const normalizedLabel = normalizeCountryLabel(label)
      if (!normalizedLabel || normalizedLabel.length < 2) return
      const key = `${countryCode}:${normalizedLabel}`
      if (seen.has(key)) return
      seen.add(key)
      entries.push({ code: countryCode, label: normalizedLabel })
    })
  })

  return entries.sort((left, right) => right.label.length - left.label.length)
}

const localizedCountryNameEntries = getLocalizedCountryNameEntries()

// 文案统一取自 locale 语言包（`proxy.*` / `setting.advanced.*`）。
// 原实现用 `locale.startsWith('zh') ? '中文' : 'English'` 内联，
// ja-JP / ru-RU 用户会直接看到英文。
const labels = computed(() => ({
  groupsTab: t('proxy.tabGroups'),
  providersTab: t('proxy.tabProviders'),
  hideUnavailable: t('proxy.filterHideUnavailable'),
  showAll: t('proxy.filterShowAll'),
  recommended: t('proxy.badgeRecommended'),
  favorites: t('proxy.badgeFavorites'),
  nodeCount: t('proxy.badgeNodeCount'),
  noProviders: t('proxy.emptyProviders'),
}))

const orderingOptions = computed(() => [
  { label: t('setting.advanced.orderNatural'), value: 'natural' },
  { label: t('setting.advanced.orderLatency'), value: 'latency' },
  { label: t('setting.advanced.orderName'), value: 'name' },
])

const filteredGroups = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return proxyStore.proxyGroups.filter((group) => {
    const matchesQuery =
      !query ||
      group.name.toLowerCase().includes(query) ||
      group.all.some((node) => node.toLowerCase().includes(query))
    return matchesQuery
  })
})

const activeGroup = computed(
  () => proxyStore.proxyGroups.find((g) => g.name === proxyStore.selectedGroup) || null,
)

const activeGroupNodes = computed(() => {
  if (!activeGroup.value) return []
  let list = proxyStore.getSortedNodesForGroup(activeGroup.value)
  const query = searchQuery.value.trim().toLowerCase()

  if (query) {
    list = list.filter((node) => node.toLowerCase().includes(query))
  }

  if (favoritesOnly.value) {
    list = list.filter((node) => proxyStore.isFavorite(node))
  }

  return list
})

const selectGroup = (name: string) => {
  proxyStore.selectedGroup = name
}

// 启动 SubscribeGroups 推送监听(测速完成 / 切换节点 / 折叠展开都会实时回写延迟)
onMounted(() => {
  proxyStore.setupGroupsDataListener()
})
onBeforeUnmount(() => {
  proxyStore.cleanupGroupsDataListener()
})

const refresh = async () => {
  try {
    await proxyStore.fetchProxies()
    message.success(t('proxy.loadSuccess'))
  } catch {
    message.error(t('proxy.loadFailedCheck'))
  }
}

const changeProxy = async (groupName: string, proxyName: string) => {
  try {
    await proxyStore.changeProxy(groupName, proxyName)
    message.success(t('proxy.switchSuccess', { group: groupName, proxy: proxyName }))
  } catch {
    message.error(t('proxy.switchErrorMessage'))
  }
}

const testActiveGroup = async () => {
  if (!activeGroup.value) return
  try {
    // 不再弹"测速完成"message:URLTest gRPC RPC 是 fire-and-forget,调完不等测速完成,
    // 真正的"测速完成"由 SubscribeGroups 推送新 frame 体现(成功的节点 urlTestDelay>0
    // 立即显示延迟;失败的 8s 后由 store 延迟标红)。用户看到延迟数字/失败标签变化
    // 就是完成信号,不需要 message 提示。
    await proxyStore.testGroupDelay(activeGroup.value.name)
  } catch {
    message.error(t('proxy.testErrorMessage'))
  }
}

const batchTest = async () => {
  try {
    await proxyStore.testAllGroups()
  } catch {
    message.error(t('proxy.testErrorMessage'))
  }
}

const switchRecommended = async (group: ProxyGroup) => {
  try {
    const recommended = await proxyStore.switchToRecommended(group)
    if (recommended) {
      message.success(t('proxy.switchSuccess', { group: group.name, proxy: recommended }))
    }
  } catch {
    message.error(t('proxy.switchErrorMessage'))
  }
}

const getNodeCountryCode = (nodeName: string) => {
  const normalizedName = stripLeadingFlagEmoji(nodeName)
  if (isInformationalNodeName(normalizedName)) return ''

  const override = COUNTRY_NAME_PATTERNS.find(([pattern]) => pattern.test(normalizedName))
  if (override) {
    return getFlagIconCountryCode(override[1])
  }

  const normalizedNodeName = normalizeCountryLabel(normalizedName)
  const localizedCountry = localizedCountryNameEntries.find((entry) =>
    normalizedNodeName.includes(entry.label),
  )
  if (localizedCountry) {
    return getFlagIconCountryCode(localizedCountry.code)
  }

  const tokens = normalizedName
    .toUpperCase()
    .split(/[^A-Z]+/)
    .filter(Boolean)
  const countryCodes = tokens.filter((token) => token === 'EU' || /^[A-Z]{2}$/.test(token))
  if (!countryCodes.length) return ''

  const preferredCode =
    countryCodes.length > 1 && PROXY_PREFIX_COUNTRY_CODES.has(countryCodes[0])
      ? countryCodes[1]
      : countryCodes[0]
  return getFlagIconCountryCode(preferredCode)
}

const getDisplayNodeName = (nodeName?: string | null) =>
  nodeName ? stripLeadingFlagEmoji(nodeName) : '-'

const getNodeFlagUrl = (nodeName?: string | null) => {
  if (!nodeName) return ''
  const countryCode = getNodeCountryCode(nodeName)
  return countryCode
    ? flagIconUrls[`../../node_modules/flag-icons/flags/4x3/${countryCode}.svg`] || ''
    : ''
}

const formatLatency = (proxyName: string) => {
  const status = proxyStore.getLatencyStatus(proxyName)
  if (status === 'ok') {
    return `${proxyStore.getLatency(proxyName)}ms`
  }
  if (status === 'failed') {
    return t('proxy.testFailed') // "测速失败" / "Test failed" / "Ошибка теста"
  }
  return t('proxy.clickToTest') // "点击测试" / "Click to test"
}

const latencyTagType = (proxyName: string) => {
  const status = proxyStore.getLatencyStatus(proxyName)
  if (activeGroup.value?.now === proxyName) return 'success' as const
  if (status === 'ok') {
    const latency = proxyStore.getLatency(proxyName)
    if (latency < 200) return 'success' as const
    if (latency < 500) return 'warning' as const
    return 'error' as const
  }
  if (status === 'failed') return 'error' as const // 红色:测过但失败
  return 'default' as const // 灰色:未测
}

// 进入代理页时,setupGroupsDataListener 内部会自检 activeConfigPath 是否变更,
// 不一致会自动 fetchProxies,无需在这里手动调用 refresh。
</script>

<style scoped>
.proxy-page {
  max-width: var(--content-max-width, 1440px);
  margin: 0 auto;
  height: 100%;
}

.split-view {
  display: flex;
  gap: 0;
  flex: 1;
  min-height: 0;
  border-radius: var(--radius-xl);
  border: 1px solid var(--panel-border);
  background: var(--panel-bg);
  box-shadow: var(--panel-shadow);
  overflow: hidden;
}

.group-sidebar {
  width: 240px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--panel-border);
  background: var(--bg-surface-2);
}

.sidebar-title {
  padding: var(--space-4) var(--space-4) var(--space-2);
  font-size: var(--text-xs);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-tertiary);
}

.sidebar-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 var(--space-2) var(--space-2);
}

.sidebar-item {
  padding: var(--space-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition-fast);
  margin-bottom: 2px;
}

.sidebar-item:hover {
  background: var(--bg-elevated);
}

.sidebar-item.active {
  background: var(--primary-soft);
}

.sidebar-item-main {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.sidebar-item-name {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-item.active .sidebar-item-name {
  color: var(--primary-color);
}

.sidebar-item-now {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: var(--space-1);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-empty {
  margin: var(--space-4) var(--space-2);
  padding: var(--space-6) var(--space-3);
}

.node-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.node-toolbar {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--panel-border);
  flex-shrink: 0;
}

.node-summary {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-4);
  font-size: var(--text-xs);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--panel-border);
  flex-shrink: 0;
  flex-wrap: wrap;
}

.summary-text {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.summary-sep {
  color: var(--text-tertiary);
  opacity: 0.5;
}

.summary-spacer {
  flex: 1;
}

.node-grid {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-3);
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: var(--space-3);
  align-content: start;
}

.node-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--panel-border);
  background: var(--bg-elevated);
  cursor: pointer;
  transition:
    transform var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.node-card:hover {
  border-color: var(--border-hover);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}

.node-card.active {
  border-color: var(--success-color);
  background: var(--success-soft);
}

.node-card-top {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.node-card-name {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.node-fav-btn {
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.node-card:hover .node-fav-btn,
.node-card.active .node-fav-btn {
  opacity: 1;
}

.node-card-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.node-flag {
  width: 1.2em;
  height: 0.85em;
  min-width: 1.2em;
  object-fit: cover;
  border-radius: 2px;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.08);
}

.node-empty {
  margin: var(--space-6) var(--space-4);
  padding: var(--space-12) var(--space-4);
}

.providers-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.providers-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.provider-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  box-shadow: var(--panel-shadow);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
}

.provider-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}

.provider-name {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
}

.provider-meta {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-top: 2px;
}

.provider-node-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--space-2);
}

.provider-node-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-sm);
  background: var(--bg-surface-2);
}

.provider-node-name {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: var(--text-xs);
  font-weight: 500;
  color: var(--text-primary);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 960px) {
  .split-view {
    flex-direction: column;
  }

  .group-sidebar {
    width: 100%;
    border-right: none;
    border-bottom: 1px solid var(--panel-border);
    max-height: 220px;
  }
}
</style>
