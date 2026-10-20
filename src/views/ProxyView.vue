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

    <!--
      代理页只放「节点 / 分组」这一个主题。
      「出站总览」已独立成页面（路由 /outbounds，侧边栏「出站」入口）——
      代理页主体是「定高 + 内部滚动」布局，同层再放有内容高度的卡片/面板
      会把 .split-view 压扁，代理组与节点列表会直接不可见。
    -->
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
import { getDisplayNodeName, getNodeFlagUrl } from '@/composables/useNodeFlag'
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
/* 页面样式已抽到 `@/assets/views/ProxyView.css`，用 @import 而非 script import：
 * 这样外部文件里的规则仍会经过 SFC 的 scoped 变换（加上 data-v-* 属性），
 * 保持与内联 <style scoped> 完全一致的隔离语义。 */
@import '@/assets/views/ProxyView.css';
</style>
