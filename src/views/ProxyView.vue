<template>
  <div class="page-container">
    <PageHeader :title="t('proxy.title')" :subtitle="t('proxy.subtitle')">
      <template #actions>
        <n-space>
          <n-button secondary @click="refresh">
            <template #icon>
              <n-icon><RefreshOutline /></n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
          <n-button type="primary" secondary :loading="proxyStore.batchTesting" @click="batchTest">
            <template #icon>
              <n-icon><SpeedometerOutline /></n-icon>
            </template>
            {{ t('proxy.batchTest') }}
          </n-button>
        </n-space>
      </template>
    </PageHeader>

    <div class="toolbar-card">
      <n-tabs v-model:value="proxyStore.viewTab" type="segment" size="small">
        <n-tab-pane name="groups" :tab="labels.groupsTab" />
        <n-tab-pane name="providers" :tab="labels.providersTab" />
      </n-tabs>

      <div class="toolbar-row">
        <n-input v-model:value="searchQuery" :placeholder="t('proxy.searchNode')" clearable>
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
        <n-select v-model:value="proxyStore.ordering" :options="orderingOptions" />
        <n-select v-model:value="proxyStore.displayMode" :options="displayModeOptions" />
        <n-button quaternary @click="favoritesOnly = !favoritesOnly">
          <template #icon>
            <n-icon>
              <Star v-if="favoritesOnly" />
              <StarOutline v-else />
            </n-icon>
          </template>
          {{ t('proxy.onlyFavorites') }}
        </n-button>
        <n-switch v-model:value="proxyStore.hideUnavailable">
          <template #checked>{{ labels.hideUnavailable }}</template>
          <template #unchecked>{{ labels.showAll }}</template>
        </n-switch>
      </div>
    </div>

    <div v-if="proxyStore.viewTab === 'groups'" class="content-grid">
      <div v-if="filteredGroups.length" class="group-list">
        <div v-for="group in filteredGroups" :key="group.name" class="group-card">
          <div class="group-header">
            <div class="group-summary">
              <div class="group-name-row">
                <h3 class="group-name">{{ group.name }}</h3>
                <n-tag size="small" round :bordered="false">{{ group.type }}</n-tag>
                <n-tag size="small" round :bordered="false" type="primary">
                  {{ group.all.length }} {{ t('proxy.nodes') }}
                </n-tag>
              </div>
              <div class="group-now">
                {{ t('proxy.currentLabel') }} {{ group.now || '-' }}
              </div>
            </div>

            <div class="group-actions">
              <n-button text @click="toggleExpanded(group.name)">
                {{ isExpanded(group.name) ? labels.collapse : labels.expand }}
              </n-button>
              <n-button text @click="testGroup(group.name)" :loading="proxyStore.groupTestingMap[group.name]">
                {{ t('proxy.testNode') }}
              </n-button>
              <n-button text @click="switchRecommended(group)">
                {{ t('proxy.autoSelect') }}
              </n-button>
            </div>
          </div>

          <div class="group-stats">
            <span>{{ labels.recommended }} {{ proxyStore.getRecommendedNode(group) || '-' }}</span>
            <span>{{ labels.favorites }} {{ group.all.filter((node) => proxyStore.isFavorite(node)).length }}</span>
          </div>

          <div v-if="isExpanded(group.name)" :class="['nodes-grid', proxyStore.displayMode]">
            <button
              v-for="node in getVisibleNodes(group)"
              :key="`${group.name}-${node}`"
              type="button"
              class="node-card"
              :class="{ active: group.now === node }"
              @click="changeProxy(group.name, node)"
            >
              <div class="node-head">
                <span class="node-name">{{ node }}</span>
                <div class="node-actions">
                  <n-button text @click.stop="proxyStore.toggleFavorite(node)">
                    <n-icon>
                      <Star v-if="proxyStore.isFavorite(node)" />
                      <StarOutline v-else />
                    </n-icon>
                  </n-button>
                </div>
              </div>

              <div class="node-meta">
                <n-tag
                  size="small"
                  round
                  :bordered="false"
                  :type="group.now === node ? 'success' : 'default'"
                >
                  {{ formatLatency(node) }}
                </n-tag>
                <span class="node-provider">
                  {{ proxyStore.proxyNodeMap[node]?.provider || proxyStore.proxyNodeMap[node]?.type || '-' }}
                </span>
              </div>

              <div class="node-footer">
                <n-button
                  size="tiny"
                  secondary
                  :loading="proxyStore.nodeTestingMap[node]"
                  @click.stop="testNode(node)"
                >
                  {{ t('proxy.testNode') }}
                </n-button>
                <n-button
                  v-if="proxyStore.isRecommendationExcluded(node)"
                  size="tiny"
                  tertiary
                  @click.stop="proxyStore.toggleRecommendationExclusion(node)"
                >
                  {{ labels.restoreRecommend }}
                </n-button>
                <n-button
                  v-else
                  size="tiny"
                  tertiary
                  @click.stop="proxyStore.toggleRecommendationExclusion(node)"
                >
                  {{ labels.skipRecommend }}
                </n-button>
              </div>
            </button>
          </div>
        </div>
      </div>

      <div v-else class="empty-state">
        <div class="empty-icon">
          <n-icon size="48"><GlobeOutline /></n-icon>
        </div>
        <h3 class="empty-title">{{ t('proxy.noProxyGroups') }}</h3>
      </div>
    </div>

    <div v-else class="content-grid">
      <div v-if="filteredProviders.length" class="providers-grid">
        <div v-for="provider in filteredProviders" :key="provider.name" class="provider-card">
          <div class="provider-head">
            <div>
              <div class="provider-name">{{ provider.name }}</div>
              <div class="provider-meta">
                {{ provider.vehicleType || '-' }} · {{ provider.behavior || '-' }}
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

          <div class="provider-stats">
            <span>{{ labels.nodeCount }} {{ provider.proxies?.length || 0 }}</span>
            <span>{{ labels.updatedAt }} {{ provider.updatedAt || provider.updateAt || '-' }}</span>
          </div>

          <div class="provider-node-list">
            <div v-for="proxy in getProviderNodes(provider)" :key="proxy.name" class="provider-node-item">
              <span class="provider-node-name">{{ proxy.name }}</span>
              <n-tag size="small" round :bordered="false">{{ formatLatency(proxy.name) }}</n-tag>
            </div>
          </div>
        </div>
      </div>

      <div v-else class="empty-state">
        <div class="empty-icon">
          <n-icon size="48"><GlobeOutline /></n-icon>
        </div>
        <h3 class="empty-title">{{ labels.noProviders }}</h3>
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
import PageHeader from '@/components/common/PageHeader.vue'
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

const labels = computed(() => ({
  groupsTab: locale.value.startsWith('zh') ? '代理组' : 'Groups',
  providersTab: locale.value.startsWith('zh') ? 'Providers' : 'Providers',
  hideUnavailable: locale.value.startsWith('zh') ? '隐藏不可用' : 'Hide unavailable',
  showAll: locale.value.startsWith('zh') ? '显示全部' : 'Show all',
  expand: locale.value.startsWith('zh') ? '展开' : 'Expand',
  collapse: locale.value.startsWith('zh') ? '收起' : 'Collapse',
  recommended: locale.value.startsWith('zh') ? '推荐节点:' : 'Recommended:',
  favorites: locale.value.startsWith('zh') ? '收藏:' : 'Favorites:',
  restoreRecommend: locale.value.startsWith('zh') ? '恢复推荐' : 'Restore',
  skipRecommend: locale.value.startsWith('zh') ? '排除推荐' : 'Skip',
  nodeCount: locale.value.startsWith('zh') ? '节点数:' : 'Nodes:',
  updatedAt: locale.value.startsWith('zh') ? '更新于:' : 'Updated:',
  noProviders: locale.value.startsWith('zh') ? '暂无 Proxy Providers' : 'No proxy providers',
}))

const orderingOptions = computed(() => [
  {
    label: locale.value.startsWith('zh') ? '原始顺序' : 'Natural',
    value: 'natural',
  },
  {
    label: locale.value.startsWith('zh') ? '按延迟' : 'Latency',
    value: 'latency',
  },
  {
    label: locale.value.startsWith('zh') ? '按名称' : 'Name',
    value: 'name',
  },
])

const displayModeOptions = computed(() => [
  {
    label: locale.value.startsWith('zh') ? '卡片' : 'Card',
    value: 'card',
  },
  {
    label: locale.value.startsWith('zh') ? '紧凑' : 'List',
    value: 'list',
  },
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

const testGroup = async (groupName: string) => {
  try {
    await proxyStore.testGroupDelay(groupName)
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

const isExpanded = (groupName: string) => proxyStore.expandedGroups.includes(groupName)
const toggleExpanded = (groupName: string) => proxyStore.toggleGroupExpanded(groupName)

const getVisibleNodes = (group: ProxyGroup) => {
  let list = proxyStore.getSortedNodesForGroup(group)
  const query = searchQuery.value.trim().toLowerCase()

  if (query) {
    list = list.filter((node) => node.toLowerCase().includes(query))
  }

  if (favoritesOnly.value) {
    list = list.filter((node) => proxyStore.isFavorite(node))
  }

  return list
}

const getProviderNodes = (provider: ProxyProvider) =>
  (provider.proxies || []).slice().sort((left, right) => left.name.localeCompare(right.name))

const formatLatency = (proxyName: string) => {
  const latency = proxyStore.getLatency(proxyName)
  return latency > 0 ? `${latency} ms` : t('proxy.clickToTest')
}

if (!proxyStore.proxyGroups.length) {
  refresh()
}
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

.toolbar-card,
.group-card,
.provider-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
}

.toolbar-card {
  padding: 16px;
}

.toolbar-row {
  margin-top: 12px;
  display: grid;
  grid-template-columns: minmax(220px, 1fr) 180px 180px auto auto;
  gap: 12px;
  align-items: center;
}

.content-grid {
  display: flex;
  flex-direction: column;
}

.group-list,
.providers-grid {
  display: grid;
  gap: 16px;
}

.group-card,
.provider-card {
  padding: 16px;
}

.group-header,
.provider-head,
.group-actions,
.group-stats,
.provider-stats {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.group-summary,
.group-actions {
  align-items: flex-start;
}

.group-name-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
}

.group-name,
.provider-name {
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
}

.group-now,
.provider-meta,
.group-stats,
.provider-stats {
  margin-top: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}

.nodes-grid {
  margin-top: 16px;
  display: grid;
  gap: 12px;
}

.nodes-grid.card {
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
}

.nodes-grid.list {
  grid-template-columns: 1fr;
}

.node-card {
  border: 1px solid var(--border-color);
  border-radius: 14px;
  background: transparent;
  padding: 12px;
  text-align: left;
  transition: border-color 0.2s ease, transform 0.2s ease;
}

.node-card.active {
  border-color: rgba(16, 185, 129, 0.45);
}

.node-card:hover {
  border-color: var(--border-hover);
  transform: translateY(-1px);
}

.node-head,
.node-meta,
.node-footer,
.provider-node-item {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  align-items: center;
}

.node-name,
.provider-node-name {
  font-weight: 600;
  color: var(--text-primary);
  word-break: break-all;
}

.node-provider {
  font-size: 12px;
  color: var(--text-tertiary);
}

.provider-node-list {
  margin-top: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
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
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
}

@media (max-width: 1100px) {
  .toolbar-row {
    grid-template-columns: 1fr;
  }

  .group-header,
  .provider-head,
  .group-stats {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
