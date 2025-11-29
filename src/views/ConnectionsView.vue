<template>
  <div class="page-container">
    <PageHeader :title="t('connections.title')" :subtitle="t('connections.subtitle')">
      <template #actions>
        <n-button
          @click="refreshConnections"
          :loading="loading"
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

    <!-- Stats -->
    <div class="stats-grid">
      <StatusCard
        v-for="stat in connectionStats"
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

    <!-- Filters -->
    <div class="filter-section">
      <div class="filter-bar">
        <n-input
          v-model:value="searchQuery"
          :placeholder="t('connections.searchPlaceholder')"
          clearable
          round
          class="search-input"
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
        
        <n-select
          v-model:value="networkFilter"
          :options="networkOptions"
          :placeholder="t('connections.networkTypeFilter')"
          clearable
          class="filter-select"
        />
        
        <n-select
          v-model:value="ruleFilter"
          :options="ruleOptions"
          :placeholder="t('connections.ruleFilter')"
          clearable
          class="filter-select"
        />
      </div>
    </div>

    <!-- Connections List -->
    <div class="connections-section">
      <n-spin :show="loading">
        <div v-if="filteredConnections.length > 0" class="connections-grid">
          <div
            v-for="conn in filteredConnections"
            :key="conn.id"
            class="connection-card"
          >
            <div class="card-header">
              <div class="header-left">
                <n-tag size="small" :type="getNetworkTagType(conn.metadata.network)" round :bordered="false">
                  {{ conn.metadata.network?.toUpperCase() || 'TCP' }}
                </n-tag>
                <span class="conn-id">#{{ getConnectionShortId(conn.id) }}</span>
              </div>
              <span class="conn-time">{{ formatConnectionTime(conn.start) }}</span>
            </div>

            <div class="card-body">
              <div class="info-row">
                <div class="info-item">
                  <span class="label">{{ t('connections.source') }}</span>
                  <span class="value" :title="getSourceText(conn)">{{ getSourceText(conn) }}</span>
                </div>
                <div class="info-item">
                  <span class="label">{{ t('connections.destination') }}</span>
                  <span class="value highlight" :title="getDestinationText(conn)">{{ getDestinationText(conn) }}</span>
                </div>
              </div>

              <div class="info-row">
                <div class="info-item">
                  <span class="label">{{ t('connections.rule') }}</span>
                  <span class="value">
                    <n-tag size="small" :type="getRuleTagType(conn.rule)" :bordered="false">
                      {{ conn.rule }}
                    </n-tag>
                  </span>
                </div>
                <div class="info-item">
                  <span class="label">{{ t('connections.traffic') }}</span>
                  <div class="traffic-stats">
                    <span class="up">
                      <n-icon size="12"><ArrowUpOutline /></n-icon>
                      {{ formatBytes(conn.upload) }}
                    </span>
                    <span class="down">
                      <n-icon size="12"><ArrowDownOutline /></n-icon>
                      {{ formatBytes(conn.download) }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Empty State -->
        <div v-else class="empty-state">
          <div class="empty-icon">
            <n-icon size="48"><LinkOutline /></n-icon>
          </div>
          <h3 class="empty-title">
            {{ searchQuery || networkFilter || ruleFilter ? t('connections.noMatchingConnections2') : t('connections.noActiveConnections') }}
          </h3>
          <n-button
            v-if="!searchQuery && !networkFilter && !ruleFilter"
            @click="refreshConnections"
            type="primary"
          >
            {{ t('connections.refreshConnections') }}
          </n-button>
          <n-button
            v-else
            @click="clearFilters"
            secondary
          >
            {{ t('rules.clearFilters') }}
          </n-button>
        </div>
      </n-spin>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  SearchOutline,
  LinkOutline,
  ArrowUpOutline,
  ArrowDownOutline,
  FilterOutline,
} from '@vicons/ionicons5'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useI18n } from 'vue-i18n'
import PageHeader from '@/components/common/PageHeader.vue'
import StatusCard from '@/components/common/StatusCard.vue'

defineOptions({
  name: 'ConnectionsView'
})

const message = useMessage()
const loading = ref(false)
const connectionStore = useConnectionStore()
const { t } = useI18n()

const searchQuery = ref('')
const networkFilter = ref(null)
const ruleFilter = ref(null)

// Interfaces
interface ConnectionMetadata {
  destinationIP: string
  destinationPort: string
  dnsMode: string
  host: string
  network: string
  processPath: string
  sourceIP: string
  sourcePort: string
  type: string
}

interface Connection {
  chains: string[]
  download: number
  id: string
  metadata: ConnectionMetadata
  rule: string
  rulePayload: string
  start: string
  upload: number
}

// Computed
const connections = computed(() => connectionStore.connections)
const connectionsTotal = computed(() => connectionStore.connectionsTotal)

const filteredConnections = computed(() => {
  return connections.value.filter((conn) => {
    const matchesSearch =
      !searchQuery.value ||
      conn.id.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      (conn.metadata.host?.toLowerCase() || '').includes(searchQuery.value.toLowerCase()) ||
      conn.metadata.destinationIP.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      conn.metadata.sourceIP.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      (conn.rule && String(conn.rule).toLowerCase().includes(searchQuery.value.toLowerCase())) ||
      (conn.metadata.processPath?.toLowerCase() || '').includes(searchQuery.value.toLowerCase())

    const matchesNetwork =
      !networkFilter.value ||
      (conn.metadata.network &&
        networkFilter.value &&
        String(conn.metadata.network).toLowerCase() === String(networkFilter.value).toLowerCase())

    const matchesRule =
      !ruleFilter.value ||
      (conn.rule &&
        ruleFilter.value &&
        String(conn.rule).toLowerCase() === String(ruleFilter.value).toLowerCase())

    return matchesSearch && matchesNetwork && matchesRule
  })
})

const networkOptions = computed(() => {
  const networks = new Set<string>()
  connections.value.forEach((conn) => {
    if (conn.metadata.network) {
      networks.add(String(conn.metadata.network).toUpperCase())
    }
  })
  return Array.from(networks).map((network) => ({ label: network, value: network.toLowerCase() }))
})

const ruleOptions = computed(() => {
  const rules = new Set<string>()
  connections.value.forEach((conn) => {
    if (conn.rule) {
      rules.add(conn.rule)
    }
  })
  return Array.from(rules).map((rule) => ({ label: rule, value: rule }))
})

const connectionStats = computed(() => [
  {
    label: t('connections.activeConnections'),
    value: connections.value.length,
    icon: LinkOutline,
    type: 'primary' as const,
  },
  {
    label: t('home.traffic.uploadTotal'),
    value: formatBytes(connectionsTotal.value.upload),
    icon: ArrowUpOutline,
    type: 'warning' as const,
  },
  {
    label: t('home.traffic.downloadTotal'),
    value: formatBytes(connectionsTotal.value.download),
    icon: ArrowDownOutline,
    type: 'success' as const,
  },
  {
    label: t('connections.matchedConnections'),
    value: filteredConnections.value.length,
    icon: FilterOutline,
    type: 'default' as const,
  },
])

// Methods
const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return (bytes / Math.pow(k, i)).toFixed(2) + ' ' + sizes[i]
}

const getConnectionShortId = (id: string): string => {
  if (!id) return 'N/A'
  return id.length > 8 ? id.substring(0, 8) + '...' : id
}

const formatConnectionTime = (timeString: string): string => {
  try {
    const date = new Date(timeString)
    const now = new Date()
    const diff = now.getTime() - date.getTime()

    if (diff < 60000) {
      return t('connections.secondsAgo', { count: Math.floor(diff / 1000) })
    } else if (diff < 3600000) {
      return t('connections.minutesAgo', { count: Math.floor(diff / 60000) })
    } else if (diff < 86400000) {
      return t('connections.hoursAgo', { count: Math.floor(diff / 3600000) })
    } else {
      return date.toLocaleDateString()
    }
  } catch (e) {
    return t('common.unknown')
  }
}

const getSourceText = (conn: Connection): string => {
  const { sourceIP, sourcePort } = conn.metadata
  return `${sourceIP}:${sourcePort}`
}

const getDestinationText = (conn: Connection): string => {
  const { destinationIP, destinationPort, host } = conn.metadata
  return host || `${destinationIP}:${destinationPort}`
}

const getNetworkTagType = (network: string): 'info' | 'warning' | 'default' => {
  if (network === 'tcp') return 'info'
  if (network === 'udp') return 'warning'
  return 'default'
}

const getRuleTagType = (rule: string): 'success' | 'error' | 'info' | 'warning' => {
  if (!rule) return 'info'
  if (rule.includes('direct')) return 'success'
  if (rule.includes('proxy')) return 'info'
  if (rule.includes('reject')) return 'error'
  return 'warning'
}

const clearFilters = () => {
  searchQuery.value = ''
  networkFilter.value = null
  ruleFilter.value = null
}

const refreshConnections = async () => {
  loading.value = true
  try {
    message.success(t('connections.refreshSuccess'))
  } catch (error) {
    message.error(t('connections.refreshError', { error: String(error) }))
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  if (!connections.value.length) {
    await connectionStore.setupEventListeners()
    refreshConnections()
  }
})

onUnmounted(() => {
  connectionStore.cleanupEventListeners()
})
</script>

<style scoped>
.page-container {
  padding: var(--layout-page-padding-y, 24px) var(--layout-page-padding-x, 32px);
  max-width: var(--layout-page-max-width, 1400px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--layout-page-gap, 24px);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--layout-row-gap, 16px);
}

.filter-section {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 16px;
}

.filter-bar {
  display: flex;
  gap: var(--layout-row-gap, 16px);
  flex-wrap: wrap;
}

.search-input {
  flex: 2;
  min-width: 200px;
}

.filter-select {
  flex: 1;
  min-width: 160px;
}

.connections-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: var(--layout-row-gap, 16px);
}

.connection-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 16px;
  transition: all 0.2s ease;
}

.connection-card:hover {
  border-color: var(--border-hover);
  transform: translateY(-2px);
  box-shadow: var(--panel-shadow);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-color);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.conn-id {
  font-family: monospace;
  font-size: 12px;
  color: var(--text-tertiary);
}

.conn-time {
  font-size: 12px;
  color: var(--text-tertiary);
}

.card-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.info-item .label {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--text-tertiary);
  letter-spacing: 0.05em;
}

.info-item .value {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.info-item .value.highlight {
  color: var(--text-primary);
  font-weight: 500;
}

.traffic-stats {
  display: flex;
  gap: 8px;
  font-size: 12px;
  font-weight: 500;
}

.traffic-stats .up {
  color: #ef4444;
  display: flex;
  align-items: center;
  gap: 2px;
}

.traffic-stats .down {
  color: #10b981;
  display: flex;
  align-items: center;
  gap: 2px;
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
  margin: 0 0 16px;
  color: var(--text-primary);
}
</style>
