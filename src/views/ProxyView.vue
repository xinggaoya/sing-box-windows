<template>
  <div class="proxy-page">
    <div class="proxy-top-bar">
      <div class="top-bar-left">
        <n-tabs v-model:value="proxyStore.viewTab" type="segment" size="small">
          <n-tab-pane name="groups" :tab="labels.groupsTab" />
          <n-tab-pane name="providers" :tab="labels.providersTab" />
        </n-tabs>
      </div>
      <div class="top-bar-right">
        <n-button size="small" secondary @click="refresh">
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          {{ t('common.refresh') }}
        </n-button>
        <n-button size="small" type="primary" secondary :loading="proxyStore.batchTesting" @click="batchTest">
          <template #icon>
            <n-icon><SpeedometerOutline /></n-icon>
          </template>
          {{ t('proxy.batchTest') }}
        </n-button>
      </div>
    </div>

    <div v-if="proxyStore.viewTab === 'groups'" class="split-view">
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
              <img v-if="getNodeFlagUrl(group.now)" class="node-flag" :src="getNodeFlagUrl(group.now)" alt="" />
              <span>{{ getDisplayNodeName(group.now) }}</span>
            </div>
          </div>
          <div v-if="!filteredGroups.length" class="sidebar-empty">
            <n-icon size="32"><GlobeOutline /></n-icon>
            <span>{{ t('proxy.noProxyGroups') }}</span>
          </div>
        </div>
      </div>

      <div class="node-panel">
        <div class="node-toolbar">
          <n-input v-model:value="searchQuery" size="small" :placeholder="t('proxy.searchNode')" clearable>
            <template #prefix>
              <n-icon><SearchOutline /></n-icon>
            </template>
          </n-input>
          <n-select v-model:value="proxyStore.ordering" size="small" :options="orderingOptions" style="width: 120px" />
          <n-button size="small" quaternary :type="favoritesOnly ? 'primary' : 'default'" @click="favoritesOnly = !favoritesOnly">
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
          <span class="summary-text">{{ labels.favorites }} {{ activeGroup.all.filter(n => proxyStore.isFavorite(n)).length }}</span>
          <div class="summary-spacer"></div>
          <n-button size="tiny" secondary @click="testActiveGroup" :loading="proxyStore.groupTestingMap[activeGroup.name]">
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
                <img v-if="getNodeFlagUrl(node)" class="node-flag" :src="getNodeFlagUrl(node)" alt="" />
                <span>{{ getDisplayNodeName(node) }}</span>
              </span>
              <n-button text size="tiny" class="node-fav-btn" @click.stop="proxyStore.toggleFavorite(node)">
                <n-icon size="14">
                  <Star v-if="proxyStore.isFavorite(node)" style="color: #f59e0b" />
                  <StarOutline v-else />
                </n-icon>
              </n-button>
            </div>
            <div class="node-card-bottom">
              <n-tag
                size="small"
                round
                :bordered="false"
                :type="latencyTagType(node)"
              >
                {{ formatLatency(node) }}
              </n-tag>
              <n-button
                size="tiny"
                quaternary
                :loading="proxyStore.nodeTestingMap[node]"
                @click.stop="testNode(node)"
              >
                <template #icon>
                  <n-icon size="13"><SpeedometerOutline /></n-icon>
                </template>
              </n-button>
            </div>
          </div>
        </div>

        <div v-else-if="activeGroup" class="node-empty">
          <n-icon size="36"><GlobeOutline /></n-icon>
          <span>{{ t('proxy.noProxyGroups') }}</span>
        </div>

        <div v-else class="node-empty">
          <n-icon size="36"><GlobeOutline /></n-icon>
          <span>{{ t('proxy.noProxyGroups') }}</span>
        </div>
      </div>
    </div>

    <div v-else class="providers-view">
      <div v-if="filteredProviders.length" class="providers-list">
        <div v-for="provider in filteredProviders" :key="provider.name" class="provider-card">
          <div class="provider-header">
            <div>
              <div class="provider-name">{{ provider.name }}</div>
              <div class="provider-meta">
                {{ provider.vehicleType || '-' }} · {{ provider.behavior || '-' }}
                · {{ labels.nodeCount }} {{ provider.proxies?.length || 0 }}
              </div>
            </div>
            <n-button
              size="small"
              secondary
              :loading="proxyStore.providerUpdatingMap[provider.name]"
              @click="refreshProvider(provider.name)"
            >
              {{ t('common.refresh') }}
            </n-button>
          </div>
          <div class="provider-node-grid">
            <div v-for="proxy in getProviderNodes(provider)" :key="proxy.name" class="provider-node-item">
              <span class="provider-node-name">
                <img v-if="getNodeFlagUrl(proxy.name)" class="node-flag" :src="getNodeFlagUrl(proxy.name)" alt="" />
                <span>{{ getDisplayNodeName(proxy.name) }}</span>
              </span>
              <n-tag size="small" round :bordered="false">{{ formatLatency(proxy.name) }}</n-tag>
            </div>
          </div>
        </div>
      </div>
      <div v-else class="node-empty">
        <n-icon size="36"><GlobeOutline /></n-icon>
        <span>{{ labels.noProviders }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
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
import { useI18n } from 'vue-i18n'
import type { ProxyProvider } from '@/types/controller'

defineOptions({
  name: 'ProxyView',
})

const { t, locale } = useI18n()
const message = useMessage()
const proxyStore = useProxyStore()
const searchQuery = ref('')
const favoritesOnly = ref(false)

const flagIconUrls = import.meta.glob('../../node_modules/flag-icons/flags/4x3/*.svg', {
  eager: true,
  query: '?url',
  import: 'default',
}) as Record<string, string>

const countryCodeAliases: Record<string, string> = {
  UK: 'GB',
}

const countryDisplayLocales = ['zh-CN', 'zh-Hans', 'zh-TW', 'zh-Hant', 'ja-JP', 'en-US']
const allCountryCodes = [
  'AC','AD','AE','AF','AG','AI','AL','AM','AO','AQ','AR','AS','AT','AU','AW','AX','AZ',
  'BA','BB','BD','BE','BF','BG','BH','BI','BJ','BL','BM','BN','BO','BQ','BR','BS','BT',
  'BV','BW','BY','BZ','CA','CC','CD','CF','CG','CH','CI','CK','CL','CM','CN','CO','CP',
  'CR','CU','CV','CW','CX','CY','CZ','DE','DG','DJ','DK','DM','DO','DZ','EA','EC','EE',
  'EG','EH','ER','ES','ET','EU','FI','FJ','FK','FM','FO','FR','GA','GB','GD','GE','GF',
  'GG','GH','GI','GL','GM','GN','GP','GQ','GR','GS','GT','GU','GW','GY','HK','HM','HN',
  'HR','HT','HU','IC','ID','IE','IL','IM','IN','IO','IQ','IR','IS','IT','JE','JM','JO',
  'JP','KE','KG','KH','KI','KM','KN','KP','KR','KW','KY','KZ','LA','LB','LC','LI','LK',
  'LR','LS','LT','LU','LV','LY','MA','MC','MD','ME','MF','MG','MH','MK','ML','MM','MN',
  'MO','MP','MQ','MR','MS','MT','MU','MV','MW','MX','MY','MZ','NA','NC','NE','NF','NG',
  'NI','NL','NO','NP','NR','NU','NZ','OM','PA','PE','PF','PG','PH','PK','PL','PM','PN',
  'PR','PS','PT','PW','PY','QA','RE','RO','RS','RU','RW','SA','SB','SC','SD','SE','SG',
  'SH','SI','SJ','SK','SL','SM','SN','SO','SR','SS','ST','SV','SX','SY','SZ','TA','TC',
  'TD','TF','TG','TH','TJ','TK','TL','TM','TN','TO','TR','TT','TV','TW','TZ','UA','UG',
  'UM','UN','US','UY','UZ','VA','VC','VE','VG','VI','VN','VU','WF','WS','XK','YE','YT',
  'ZA','ZM','ZW',
]

const countryNameOverrides: Array<[RegExp, string]> = [
  [/东京|東京|大阪/i, 'JP'],
  [/香港|hong\s*kong/i, 'HK'],
  [/台湾|台灣|taiwan/i, 'TW'],
  [/美国|美國|洛杉矶|洛杉磯|西雅图|西雅圖|纽约|紐約|america|united\s*states/i, 'US'],
  [/英国|英國|伦敦|倫敦|britain|united\s*kingdom/i, 'GB'],
  [/首尔|首爾|korea/i, 'KR'],
  [/澳洲/i, 'AU'],
  [/沙特阿拉伯|沙特|saudi/i, 'SA'],
  [/阿联酋|阿聯酋|uae|united\s*arab\s*emirates/i, 'AE'],
]

const proxyPrefixCountryCodes = new Set(['CF', 'CL', 'SS'])
const informationalNodePatterns = [
  /^剩余流量[:：]/,
  /^距离下次重置剩余[:：]/,
  /^套餐到期[:：]/,
  /^官网[:：]/,
  /^星云[:：]/,
  /^https?:\/\//i,
]

const getFlagIconCountryCode = (code: string) => {
  if (code === 'EU') return 'eu'
  const normalizedCode = countryCodeAliases[code] || code
  if (!/^[A-Z]{2}$/.test(normalizedCode)) return ''
  return normalizedCode.toLowerCase()
}

const stripLeadingFlagEmoji = (nodeName: string) =>
  nodeName.replace(/^[\u{1f1e6}-\u{1f1ff}]{2}\s*/u, '').trim()

const isInformationalNodeName = (nodeName: string) =>
  informationalNodePatterns.some((pattern) => pattern.test(stripLeadingFlagEmoji(nodeName)))

const normalizeCountryLabel = (value: string) =>
  value
    .toLowerCase()
    .replace(/[\s·・,，.。()（）[\]【】'’"“”_-]/g, '')
    .trim()

const getLocalizedCountryNameEntries = () => {
  const entries: Array<{ code: string; label: string }> = []
  const seen = new Set<string>()

  countryDisplayLocales.forEach((displayLocale) => {
    const displayNames = new Intl.DisplayNames([displayLocale], { type: 'region' })
    allCountryCodes.forEach((countryCode) => {
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

const labels = computed(() => ({
  groupsTab: locale.value.startsWith('zh') ? '代理组' : 'Groups',
  providersTab: locale.value.startsWith('zh') ? 'Providers' : 'Providers',
  hideUnavailable: locale.value.startsWith('zh') ? '隐藏不可用' : 'Hide',
  showAll: locale.value.startsWith('zh') ? '显示全部' : 'Show',
  recommended: locale.value.startsWith('zh') ? '推荐:' : 'Rec:',
  favorites: locale.value.startsWith('zh') ? '收藏:' : 'Fav:',
  nodeCount: locale.value.startsWith('zh') ? '节点:' : 'Nodes:',
  noProviders: locale.value.startsWith('zh') ? '暂无 Proxy Providers' : 'No proxy providers',
}))

const orderingOptions = computed(() => [
  { label: locale.value.startsWith('zh') ? '原始' : 'Natural', value: 'natural' },
  { label: locale.value.startsWith('zh') ? '延迟' : 'Latency', value: 'latency' },
  { label: locale.value.startsWith('zh') ? '名称' : 'Name', value: 'name' },
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

const activeGroup = computed(() =>
  proxyStore.proxyGroups.find((g) => g.name === proxyStore.selectedGroup) || null,
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

const filteredProviders = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return proxyStore.proxyProviders.filter((provider) => {
    return (
      !query ||
      provider.name.toLowerCase().includes(query) ||
      provider.proxies?.some((proxy) => proxy.name.toLowerCase().includes(query))
    )
  })
})

const selectGroup = (name: string) => {
  proxyStore.selectedGroup = name
}

const refresh = async () => {
  try {
    await proxyStore.fetchProxies()
    message.success(t('proxy.loadSuccess'))
  } catch (error) {
    message.error(t('proxy.loadFailedCheck'))
  }
}

const changeProxy = async (groupName: string, proxyName: string) => {
  try {
    await proxyStore.changeProxy(groupName, proxyName)
    message.success(t('proxy.switchSuccess', { group: groupName, proxy: proxyName }))
  } catch (error) {
    message.error(t('proxy.switchErrorMessage'))
  }
}

const testNode = async (proxyName: string) => {
  try {
    await proxyStore.testNodeDelay(proxyName)
  } catch (error) {
    message.error(t('proxy.testFailed'))
  }
}

const testActiveGroup = async () => {
  if (!activeGroup.value) return
  try {
    await proxyStore.testGroupDelay(activeGroup.value.name)
    message.success(t('proxy.groupTestComplete'))
  } catch (error) {
    message.error(t('proxy.testErrorMessage'))
  }
}

const batchTest = async () => {
  try {
    await proxyStore.testAllGroups()
    message.success(t('proxy.batchTestComplete'))
  } catch (error) {
    message.error(t('proxy.testErrorMessage'))
  }
}

const switchRecommended = async (group: ProxyGroup) => {
  try {
    const recommended = await proxyStore.switchToRecommended(group)
    if (recommended) {
      message.success(t('proxy.switchSuccess', { group: group.name, proxy: recommended }))
    }
  } catch (error) {
    message.error(t('proxy.switchErrorMessage'))
  }
}

const refreshProvider = async (providerName: string) => {
  try {
    await proxyStore.updateProvider(providerName)
    message.success(providerName)
  } catch (error) {
    message.error(String(error))
  }
}

const getProviderNodes = (provider: ProxyProvider) =>
  (provider.proxies || []).slice().sort((left, right) => left.name.localeCompare(right.name))

const getNodeCountryCode = (nodeName: string) => {
  const normalizedName = stripLeadingFlagEmoji(nodeName)
  if (isInformationalNodeName(normalizedName)) return ''

  const override = countryNameOverrides.find(([pattern]) => pattern.test(normalizedName))
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
    countryCodes.length > 1 && proxyPrefixCountryCodes.has(countryCodes[0])
      ? countryCodes[1]
      : countryCodes[0]
  return getFlagIconCountryCode(preferredCode)
}

const getDisplayNodeName = (nodeName?: string | null) =>
  nodeName ? stripLeadingFlagEmoji(nodeName) : '-'

const getNodeFlagUrl = (nodeName?: string | null) => {
  if (!nodeName) return ''
  const countryCode = getNodeCountryCode(nodeName)
  return countryCode ? flagIconUrls[`../../node_modules/flag-icons/flags/4x3/${countryCode}.svg`] || '' : ''
}

const formatLatency = (proxyName: string) => {
  const latency = proxyStore.getLatency(proxyName)
  return latency > 0 ? `${latency}ms` : t('proxy.clickToTest')
}

const latencyTagType = (proxyName: string) => {
  const latency = proxyStore.getLatency(proxyName)
  if (activeGroup.value?.now === proxyName) return 'success' as const
  if (latency <= 0) return 'default' as const
  if (latency < 200) return 'success' as const
  if (latency < 500) return 'warning' as const
  return 'error' as const
}

if (!proxyStore.proxyGroups.length) {
  refresh()
}
</script>

<style scoped>
.proxy-page {
  padding: var(--layout-page-padding-y, 16px) var(--layout-page-padding-x, 24px);
  max-width: var(--layout-page-max-width, 1400px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}

.proxy-top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-shrink: 0;
}

.top-bar-left {
  flex-shrink: 0;
}

.top-bar-right {
  display: flex;
  gap: 8px;
}

.split-view {
  display: flex;
  gap: 0;
  flex: 1;
  min-height: 0;
  border-radius: 14px;
  border: 1px solid var(--panel-border);
  background: var(--panel-bg);
  overflow: hidden;
}

.group-sidebar {
  width: 240px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--panel-border);
  background: var(--bg-secondary);
}

.sidebar-title {
  padding: 14px 16px 8px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-tertiary);
}

.sidebar-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 8px;
}

.sidebar-item {
  padding: 10px 12px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.15s ease;
  margin-bottom: 2px;
}

.sidebar-item:hover {
  background: var(--bg-tertiary);
}

.sidebar-item.active {
  background: rgba(99, 102, 241, 0.08);
}

.sidebar-item-main {
  display: flex;
  align-items: center;
  gap: 8px;
}

.sidebar-item-name {
  font-size: 13px;
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
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 16px;
  gap: 12px;
  color: var(--text-tertiary);
  font-size: 13px;
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
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--panel-border);
  flex-shrink: 0;
}

.node-summary {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  font-size: 12px;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--panel-border);
  flex-shrink: 0;
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
  padding: 12px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
  align-content: start;
}

.node-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: 12px;
  border: 1px solid var(--panel-border);
  background: var(--bg-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.node-card:hover {
  border-color: var(--border-hover);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.node-card.active {
  border-color: rgba(16, 185, 129, 0.45);
  background: rgba(16, 185, 129, 0.04);
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
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.node-fav-btn {
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.node-card:hover .node-fav-btn {
  opacity: 1;
}

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
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 16px;
  gap: 12px;
  color: var(--text-tertiary);
  font-size: 13px;
}

.providers-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.providers-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.provider-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  padding: 16px;
}

.provider-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.provider-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.provider-meta {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.provider-node-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

.provider-node-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 8px;
  background: var(--bg-tertiary);
}

.provider-node-name {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 900px) {
  .split-view {
    flex-direction: column;
  }

  .group-sidebar {
    width: 100%;
    border-right: none;
    border-bottom: 1px solid var(--panel-border);
    max-height: 200px;
  }

  .proxy-top-bar {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
