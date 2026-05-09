<template>
  <div class="page-container">
    <PageHeader :title="t('connections.title')" :subtitle="t('connections.subtitle')">
      <template #actions>
        <n-space>
          <n-button secondary @click="refreshConnections">
            <template #icon>
              <n-icon><RefreshOutline /></n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
          <n-button secondary @click="connectionStore.togglePaused()">
            <template #icon>
              <n-icon>
                <PauseOutline v-if="!connectionStore.paused" />
                <PlayOutline v-else />
              </n-icon>
            </template>
            {{ connectionStore.paused ? proxyLabels.resume : proxyLabels.pause }}
          </n-button>
          <n-button
            type="error"
            secondary
            :loading="connectionStore.isClosingAll"
            @click="closeAll"
          >
            <template #icon>
              <n-icon><CloseOutline /></n-icon>
            </template>
            {{ proxyLabels.closeAll }}
          </n-button>
        </n-space>
      </template>
    </PageHeader>

    <div class="toolbar-card">
      <div class="toolbar-row">
        <n-tabs
          :value="connectionStore.activeTab"
          type="segment"
          size="small"
          @update:value="updateActiveTab"
        >
          <n-tab-pane name="active" :tab="proxyLabels.active" />
          <n-tab-pane name="closed" :tab="proxyLabels.closed" />
        </n-tabs>

        <n-input
          v-model:value="connectionStore.searchQuery"
          :placeholder="t('connections.searchPlaceholder')"
          clearable
          class="search-input"
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>

        <n-select
          v-model:value="connectionStore.sourceIPFilter"
          :options="sourceIpOptions"
          clearable
          :placeholder="proxyLabels.sourceFilter"
          class="source-select"
        />

        <n-select
          v-model:value="connectionStore.sortKey"
          :options="sortOptions"
          class="sort-select"
        />

        <n-select
          v-model:value="connectionStore.groupingKey"
          :options="groupingOptions"
          clearable
          class="sort-select"
          :placeholder="proxyLabels.grouping"
        />

        <n-button quaternary @click="connectionStore.sortDesc = !connectionStore.sortDesc">
          <template #icon>
            <n-icon>
              <ArrowDownOutline v-if="connectionStore.sortDesc" />
              <ArrowUpOutline v-else />
            </n-icon>
          </template>
          {{ proxyLabels.sortOrder }}
        </n-button>
      </div>

      <div class="stats-row">
        <n-tag size="small" round :bordered="false" type="primary">
          {{ t('connections.activeConnections') }}: {{ connectionStore.activeConnections.length }}
        </n-tag>
        <n-tag size="small" round :bordered="false">
          {{ proxyLabels.closed }}: {{ connectionStore.closedConnections.length }}
        </n-tag>
        <n-tag size="small" round :bordered="false" type="warning">
          ↑ {{ formatBytes(connectionStore.connectionsTotal.upload) }}
        </n-tag>
        <n-tag size="small" round :bordered="false" type="success">
          ↓ {{ formatBytes(connectionStore.connectionsTotal.download) }}
        </n-tag>
        <n-tag size="small" round :bordered="false" type="default">
          {{ proxyLabels.quickFilter }}: {{ connectionStore.quickFilterEnabled ? labelsOnOff.on : labelsOnOff.off }}
        </n-tag>
      </div>
    </div>

    <div v-if="groupedRows.length" class="table-card">
      <template v-for="group in groupedRows" :key="group.key">
        <div v-if="group.type === 'group'" class="group-row">
          <div class="group-title">
            <span>{{ group.key }}</span>
            <n-tag size="tiny" round>{{ group.count }}</n-tag>
          </div>
        </div>

        <button
          v-for="connection in group.connections"
          :key="connection.id"
          type="button"
          class="connection-row"
          @click="selectedConnection = connection"
        >
          <div class="row-top">
            <div class="row-title">
              <n-tag size="small" :bordered="false" round :type="getNetworkType(connection)">
                {{ (connection.metadata.network || 'tcp').toUpperCase() }}
              </n-tag>
              <span class="destination">{{ getDestinationText(connection) }}</span>
            </div>
            <div class="row-actions">
              <span class="connect-time">{{ formatTimeAgo(connection.start) }}</span>
              <n-button
                text
                type="error"
                :loading="connectionStore.closingMap[connection.id]"
                @click.stop="closeOne(connection.id)"
              >
                {{ proxyLabels.close }}
              </n-button>
            </div>
          </div>

          <div class="row-grid">
            <div>
              <span class="cell-label">{{ t('connections.source') }}</span>
              <span class="cell-value">{{ getSourceText(connection) }}</span>
            </div>
            <div>
              <span class="cell-label">{{ t('connections.rule') }}</span>
              <span class="cell-value">{{ getRuleText(connection) }}</span>
            </div>
            <div>
              <span class="cell-label">Chain</span>
              <span class="cell-value">{{ connection.chains.join(' > ') || '-' }}</span>
            </div>
            <div>
              <span class="cell-label">{{ t('connections.traffic') }}</span>
              <span class="cell-value">
                ↑ {{ formatSpeed(connection.uploadSpeed || 0) }} / ↓
                {{ formatSpeed(connection.downloadSpeed || 0) }}
              </span>
            </div>
          </div>
        </button>
      </template>
    </div>

    <div v-else class="empty-state">
      <div class="empty-icon">
        <n-icon size="48"><LinkOutline /></n-icon>
      </div>
      <h3 class="empty-title">
        {{ connectionStore.activeTab === 'active' ? t('connections.noActiveConnections') : proxyLabels.noClosed }}
      </h3>
    </div>

    <n-modal v-model:show="detailVisible" preset="card" :title="proxyLabels.detailTitle" style="width: 720px">
      <div v-if="selectedConnection" class="detail-grid">
        <div><strong>ID</strong><span>{{ selectedConnection.id }}</span></div>
        <div><strong>{{ t('connections.rule') }}</strong><span>{{ getRuleText(selectedConnection) }}</span></div>
        <div><strong>{{ t('connections.source') }}</strong><span>{{ getSourceText(selectedConnection) }}</span></div>
        <div><strong>{{ t('connections.destination') }}</strong><span>{{ getDestinationText(selectedConnection) }}</span></div>
        <div><strong>Process</strong><span>{{ selectedConnection.metadata.process || selectedConnection.metadata.processPath || '-' }}</span></div>
        <div><strong>Inbound</strong><span>{{ selectedConnection.metadata.inboundName || selectedConnection.metadata.inboundUser || '-' }}</span></div>
        <div><strong>Network</strong><span>{{ selectedConnection.metadata.network || '-' }}</span></div>
        <div><strong>Type</strong><span>{{ selectedConnection.metadata.type || '-' }}</span></div>
        <div><strong>Sniff Host</strong><span>{{ selectedConnection.metadata.sniffHost || '-' }}</span></div>
        <div><strong>Remote</strong><span>{{ selectedConnection.metadata.remoteDestination || '-' }}</span></div>
        <div><strong>Upload</strong><span>{{ formatBytes(selectedConnection.upload) }}</span></div>
        <div><strong>Download</strong><span>{{ formatBytes(selectedConnection.download) }}</span></div>
      </div>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useMessage } from 'naive-ui'
import {
  ArrowDownOutline,
  ArrowUpOutline,
  CloseOutline,
  LinkOutline,
  PauseOutline,
  PlayOutline,
  RefreshOutline,
  SearchOutline,
} from '@vicons/ionicons5'
import PageHeader from '@/components/common/PageHeader.vue'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useI18n } from 'vue-i18n'
import type { ConnectionItem } from '@/types/events'
import { formatBytes, formatSpeed } from '@/utils'

defineOptions({
  name: 'ConnectionsView',
})

const { t, locale } = useI18n()
const message = useMessage()
const connectionStore = useConnectionStore()
const selectedConnection = ref<ConnectionItem | null>(null)

const proxyLabels = computed(() => ({
  active: locale.value.startsWith('zh') ? '活跃连接' : 'Active',
  closed: locale.value.startsWith('zh') ? '已关闭' : 'Closed',
  pause: locale.value.startsWith('zh') ? '暂停更新' : 'Pause',
  resume: locale.value.startsWith('zh') ? '恢复更新' : 'Resume',
  close: locale.value.startsWith('zh') ? '关闭' : 'Close',
  closeAll: locale.value.startsWith('zh') ? '关闭全部连接' : 'Close All',
  sourceFilter: locale.value.startsWith('zh') ? '来源 IP' : 'Source IP',
  sortOrder: locale.value.startsWith('zh') ? '顺序' : 'Order',
  grouping: locale.value.startsWith('zh') ? '分组' : 'Grouping',
  quickFilter: locale.value.startsWith('zh') ? '快速筛选' : 'Quick filter',
  detailTitle: locale.value.startsWith('zh') ? '连接详情' : 'Connection Details',
  noClosed: locale.value.startsWith('zh') ? '暂无已关闭连接' : 'No closed connections',
}))

const labelsOnOff = computed(() => ({
  on: locale.value.startsWith('zh') ? '开' : 'On',
  off: locale.value.startsWith('zh') ? '关' : 'Off',
}))

const detailVisible = computed({
  get: () => !!selectedConnection.value,
  set: (value: boolean) => {
    if (!value) {
      selectedConnection.value = null
    }
  },
})

const sourceIpOptions = computed(() => {
  const values = new Set<string>()
  connectionStore.searchableConnections.forEach((connection) => {
    if (connection.metadata.sourceIP) {
      values.add(connection.metadata.sourceIP)
    }
  })
  return Array.from(values).map((value) => ({ label: value, value }))
})

const sortOptions = computed(() => {
  const labels: Record<string, string> = {
    start: locale.value.startsWith('zh') ? '连接时间' : 'Start Time',
    download: locale.value.startsWith('zh') ? '下载总量' : 'Download',
    upload: locale.value.startsWith('zh') ? '上传总量' : 'Upload',
    downloadSpeed: locale.value.startsWith('zh') ? '下载速度' : 'Download Speed',
    uploadSpeed: locale.value.startsWith('zh') ? '上传速度' : 'Upload Speed',
    host: locale.value.startsWith('zh') ? '目标地址' : 'Host',
    process: locale.value.startsWith('zh') ? '进程' : 'Process',
    rule: locale.value.startsWith('zh') ? '规则' : 'Rule',
  }

  return Object.entries(labels).map(([value, label]) => ({ label, value }))
})

const groupingOptions = computed(() => [
  { label: locale.value.startsWith('zh') ? '按规则' : 'Rule', value: 'rule' },
  { label: locale.value.startsWith('zh') ? '按进程' : 'Process', value: 'process' },
  { label: locale.value.startsWith('zh') ? '按目标' : 'Destination', value: 'host' },
  { label: locale.value.startsWith('zh') ? '按来源 IP' : 'Source IP', value: 'sourceIP' },
])

const filteredConnections = computed(() => {
  const query = connectionStore.searchQuery.trim().toLowerCase()
  return connectionStore.searchableConnections.filter((connection) => {
    const matchesQuery =
      !query ||
      connection.id.toLowerCase().includes(query) ||
      getDestinationText(connection).toLowerCase().includes(query) ||
      getSourceText(connection).toLowerCase().includes(query) ||
      getRuleText(connection).toLowerCase().includes(query) ||
      (connection.metadata.process || connection.metadata.processPath || '').toLowerCase().includes(query)

    const matchesSource =
      !connectionStore.sourceIPFilter || connection.metadata.sourceIP === connectionStore.sourceIPFilter

    const matchesQuickFilter =
      !connectionStore.quickFilterEnabled ||
      !getRuleText(connection).toLowerCase().includes('direct')

    return matchesQuery && matchesSource && matchesQuickFilter
  })
})

const sortedConnections = computed(() => {
  const list = [...filteredConnections.value]
  const factor = connectionStore.sortDesc ? -1 : 1

  return list.sort((left, right) => {
    const leftValue = getSortValue(left, connectionStore.sortKey)
    const rightValue = getSortValue(right, connectionStore.sortKey)

    if (typeof leftValue === 'number' && typeof rightValue === 'number') {
      return (leftValue - rightValue) * factor
    }

    return String(leftValue).localeCompare(String(rightValue)) * factor
  })
})

const groupedRows = computed(() => {
  const grouping = connectionStore.groupingKey
  if (!grouping) {
    return [
      {
        type: 'group' as const,
        key: '',
        count: sortedConnections.value.length,
        connections: sortedConnections.value,
      },
    ]
  }

  const groups = new Map<string, ConnectionItem[]>()
  sortedConnections.value.forEach((connection) => {
    const key = String(getGroupValue(connection, grouping) || '-')
    const list = groups.get(key) || []
    list.push(connection)
    groups.set(key, list)
  })

  return Array.from(groups.entries()).map(([key, connections]) => ({
    type: 'group' as const,
    key,
    count: connections.length,
    connections,
  }))
})

const refreshConnections = async () => {
  await connectionStore.setupEventListeners()
  message.success(t('connections.refreshSuccess'))
}

const closeOne = async (id: string) => {
  try {
    await connectionStore.closeConnection(id)
    message.success(proxyLabels.value.close)
  } catch (error) {
    message.error(String(error))
  }
}

const closeAll = async () => {
  try {
    await connectionStore.closeAllConnections()
    message.success(proxyLabels.value.closeAll)
  } catch (error) {
    message.error(String(error))
  }
}

const updateActiveTab = (value: string) => {
  if (value === 'active' || value === 'closed') {
    connectionStore.activeTab = value
  }
}

const getSortValue = (connection: ConnectionItem, key: string) => {
  switch (key) {
    case 'download':
      return connection.download
    case 'upload':
      return connection.upload
    case 'downloadSpeed':
      return connection.downloadSpeed || 0
    case 'uploadSpeed':
      return connection.uploadSpeed || 0
    case 'host':
      return getDestinationText(connection)
    case 'process':
      return connection.metadata.process || connection.metadata.processPath || ''
    case 'rule':
      return getRuleText(connection)
    default:
      return new Date(connection.start).getTime()
  }
}

const getGroupValue = (connection: ConnectionItem, key: string) => {
  switch (key) {
    case 'rule':
      return connection.rule
    case 'process':
      return connection.metadata.process || connection.metadata.processPath
    case 'host':
      return getDestinationText(connection)
    case 'sourceIP':
      return connection.metadata.sourceIP
    default:
      return getSortValue(connection, key)
  }
}

const getSourceText = (connection: ConnectionItem) =>
  `${connection.metadata.sourceIP || '-'}:${connection.metadata.sourcePort || '-'}`

const getDestinationText = (connection: ConnectionItem) =>
  connection.metadata.remoteDestination ||
  connection.metadata.host ||
  `${connection.metadata.destinationIP || '-'}:${connection.metadata.destinationPort || '-'}`

const getRuleText = (connection: ConnectionItem) =>
  connection.rulePayload ? `${connection.rule} : ${connection.rulePayload}` : connection.rule || '-'

const getNetworkType = (connection: ConnectionItem) => {
  const network = (connection.metadata.network || '').toLowerCase()
  if (network === 'udp') return 'warning'
  if (network === 'tcp') return 'info'
  return 'default'
}

const formatTimeAgo = (time: string) => {
  const diff = Date.now() - new Date(time).getTime()
  if (diff < 60_000) return t('connections.secondsAgo', { count: Math.max(1, Math.floor(diff / 1000)) })
  if (diff < 3_600_000) return t('connections.minutesAgo', { count: Math.floor(diff / 60_000) })
  if (diff < 86_400_000) return t('connections.hoursAgo', { count: Math.floor(diff / 3_600_000) })
  return new Date(time).toLocaleString()
}

watch(
  () => connectionStore.activeTab,
  () => {
    selectedConnection.value = null
  },
)
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
.table-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 16px;
}

.toolbar-row {
  display: grid;
  grid-template-columns: auto minmax(220px, 1fr) 180px 180px 180px auto;
  gap: 12px;
  align-items: center;
}

.search-input,
.source-select,
.sort-select {
  width: 100%;
}

.stats-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 12px;
}

.group-row {
  padding: 8px 0;
}

.group-title {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

.connection-row {
  width: 100%;
  border: 1px solid var(--border-color);
  background: transparent;
  border-radius: 14px;
  padding: 14px;
  margin-top: 10px;
  text-align: left;
  transition: border-color 0.2s ease, transform 0.2s ease;
}

.connection-row:hover {
  border-color: var(--border-hover);
  transform: translateY(-1px);
}

.row-top {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.row-title,
.row-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.destination {
  font-weight: 600;
  color: var(--text-primary);
}

.connect-time {
  font-size: 12px;
  color: var(--text-tertiary);
}

.row-grid {
  margin-top: 12px;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.cell-label {
  display: block;
  font-size: 11px;
  text-transform: uppercase;
  color: var(--text-tertiary);
  margin-bottom: 4px;
}

.cell-value {
  display: block;
  color: var(--text-secondary);
  word-break: break-all;
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
  margin: 0;
  color: var(--text-primary);
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.detail-grid div {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-grid span {
  word-break: break-all;
}

@media (max-width: 960px) {
  .toolbar-row {
    grid-template-columns: 1fr;
  }

  .row-grid,
  .detail-grid {
    grid-template-columns: 1fr;
  }

  .row-top {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
