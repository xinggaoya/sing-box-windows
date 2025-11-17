<template>
  <div class="page-shell connections-page" :style="pageThemeStyle">
    <section class="page-hero">
      <div class="hero-row">
        <div class="hero-left">
          <div class="hero-icon">
            <n-icon size="26">
              <LinkOutline />
            </n-icon>
          </div>
          <div class="hero-meta">
            <p class="hero-subtitle">{{ t('connections.subtitle') }}</p>
            <h2 class="hero-title">{{ t('connections.title') }}</h2>
          </div>
        </div>
        <div class="hero-actions">
          <n-button
            @click="refreshConnections"
            :loading="loading"
            type="primary"
            size="large"
          >
            <template #icon>
              <n-icon size="18">
                <RefreshOutline />
              </n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
        </div>
      </div>
      <div class="hero-stats">
        <div
          v-for="stat in connectionStats"
          :key="stat.label"
          class="stat-card"
          :data-accent="stat.accent"
        >
          <div class="stat-icon">
            <n-icon :size="20">
              <component :is="stat.icon" />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stat.value }}</div>
            <div class="stat-label">{{ stat.label }}</div>
          </div>
        </div>
      </div>
    </section>

    <section class="page-section">
      <n-card class="surface-card filter-panel" :bordered="false">
        <div class="filter-content">
          <div class="filter-row">
            <n-input
              v-model:value="searchQuery"
              :placeholder="t('connections.searchPlaceholder')"
              clearable
              size="large"
              class="search-input"
            >
              <template #prefix>
                <n-icon size="16">
                  <SearchOutline />
                </n-icon>
              </template>
            </n-input>
          </div>
          <div class="filter-row">
            <n-select
              v-model:value="networkFilter"
              :options="networkOptions"
              :placeholder="t('connections.networkTypeFilter')"
              clearable
              size="large"
              class="filter-select"
            />
            <n-select
              v-model:value="ruleFilter"
              :options="ruleOptions"
              :placeholder="t('connections.ruleFilter')"
              clearable
              size="large"
              class="filter-select"
            />
          </div>
          <div class="active-filters" v-if="searchQuery || networkFilter || ruleFilter">
            <n-tag v-if="searchQuery" size="small" round closable @close="searchQuery = ''">
              {{ t('common.search') }}: {{ searchQuery }}
            </n-tag>
            <n-tag v-if="networkFilter" size="small" round closable @close="networkFilter = null">
              {{ t('connections.networkTypeFilter') }}: {{ networkFilter }}
            </n-tag>
            <n-tag v-if="ruleFilter" size="small" round closable @close="ruleFilter = null">
              {{ t('connections.ruleFilter') }}: {{ ruleFilter }}
            </n-tag>
          </div>
        </div>
      </n-card>

      <n-card class="surface-card connections-card" :bordered="false">
        <n-spin :show="loading">
          <div v-if="filteredConnections.length > 0" class="connections-grid">
            <div
            v-for="(conn, index) in filteredConnections"
            :key="conn.id"
            class="connection-item"
            :class="{ 'connection-highlight': isConnectionHighlighted(conn) }"
          >
            <!-- 连接头部 -->
            <div class="connection-header">
              <div class="connection-id">
                <n-tag size="small" :type="getNetworkTagType(conn.metadata.network)">
                  {{ getConnectionShortId(conn.id) }}
                </n-tag>
              </div>
              <div class="connection-time">
                {{ formatConnectionTime(conn.start) }}
              </div>
            </div>

            <!-- 连接详情 -->
            <div class="connection-details">
              <div class="connection-row">
                <div class="detail-item">
                  <div class="detail-label">{{ t('connections.source') }}</div>
                  <div class="detail-value" :title="getSourceText(conn)">
                    {{ getSourceText(conn) }}
                  </div>
                </div>
                <div class="detail-item">
                  <div class="detail-label">{{ t('connections.destination') }}</div>
                  <div class="detail-value" :title="getDestinationText(conn)">
                    {{ getDestinationText(conn) }}
                  </div>
                </div>
              </div>

              <div class="connection-row">
                <div class="detail-item">
                  <div class="detail-label">{{ t('connections.rule') }}</div>
                  <div class="detail-value">
                    <n-tag size="small" :type="getRuleTagType(conn.rule)">
                      {{ conn.rule }}
                    </n-tag>
                  </div>
                </div>
                <div class="detail-item">
                  <div class="detail-label">{{ t('connections.traffic') }}</div>
                  <div class="detail-value traffic-value">
                    <span class="upload">↑{{ formatBytes(conn.upload) }}</span>
                    <span class="download">↓{{ formatBytes(conn.download) }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 连接状态指示器 -->
            <div class="connection-indicator" :class="getNetworkClass(conn.metadata.network)"></div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else class="empty-state">
          <div class="empty-icon">
            <n-icon size="48">
              <LinkOutline />
            </n-icon>
          </div>
          <div class="empty-title">
            {{
              searchQuery || networkFilter || ruleFilter
                ? t('connections.noMatchingConnections2')
                : t('connections.noActiveConnections')
            }}
          </div>
          <div class="empty-desc">
            {{
              searchQuery || networkFilter || ruleFilter
                ? t('connections.adjustSearchOrFilters')
                : t('connections.refreshConnections')
            }}
          </div>
          <n-button
            v-if="!searchQuery && !networkFilter && !ruleFilter"
            @click="refreshConnections"
            type="primary"
            size="large"
            class="empty-btn"
          >
            <template #icon>
              <n-icon size="18">
                <RefreshOutline />
              </n-icon>
            </template>
            {{ t('connections.refreshConnections') }}
          </n-button>
        </div>
      </n-spin>
      </n-card>
    </section>
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
import { useThemeStore } from '@/stores/app/ThemeStore'
import { usePageTheme } from '@/composables/usePageTheme'

defineOptions({
  name: 'ConnectionsView'
})

const message = useMessage()
const loading = ref(false)
const connectionStore = useConnectionStore()
const { t } = useI18n()
const themeStore = useThemeStore()
const pageThemeStyle = usePageTheme(themeStore)

// 搜索和筛选
const searchQuery = ref('')
const networkFilter = ref(null)
const ruleFilter = ref(null)

// 使用计算属性来获取连接信息
const connections = computed(() => connectionStore.connections)
const connectionsTotal = computed(() => connectionStore.connectionsTotal)

// 定义连接数据接口
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

// 筛选后的连接列表
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

// 网络类型选项
const networkOptions = computed(() => {
  const networks = new Set<string>()
  connections.value.forEach((conn) => {
    if (conn.metadata.network) {
      networks.add(String(conn.metadata.network).toUpperCase())
    }
  })
  return Array.from(networks).map((network) => ({ label: network, value: network.toLowerCase() }))
})

// 规则选项
const ruleOptions = computed(() => {
  const rules = new Set<string>()
  connections.value.forEach((conn) => {
    if (conn.rule) {
      rules.add(conn.rule)
    }
  })
  return Array.from(rules).map((rule) => ({ label: rule, value: rule }))
})

// 格式化字节大小的函数
const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return (bytes / Math.pow(k, i)).toFixed(2) + ' ' + sizes[i]
}

const connectionStats = computed(() => [
  {
    label: t('connections.activeConnections'),
    value: connections.value.length,
    icon: LinkOutline,
    accent: 'purple',
  },
  {
    label: t('home.traffic.uploadTotal'),
    value: formatBytes(connectionsTotal.value.upload),
    icon: ArrowUpOutline,
    accent: 'pink',
  },
  {
    label: t('home.traffic.downloadTotal'),
    value: formatBytes(connectionsTotal.value.download),
    icon: ArrowDownOutline,
    accent: 'blue',
  },
  {
    label: t('connections.matchedConnections'),
    value: filteredConnections.value.length,
    icon: FilterOutline,
    accent: 'amber',
  },
])

// 辅助方法
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

const getNetworkClass = (network: string): string => {
  if (network === 'tcp') return 'network-tcp'
  if (network === 'udp') return 'network-udp'
  return 'network-other'
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

const isConnectionHighlighted = (conn: Connection): boolean => {
  if (!searchQuery.value) return false

  const searchText = searchQuery.value.toLowerCase()
  const sourceText = getSourceText(conn).toLowerCase()
  const destText = getDestinationText(conn).toLowerCase()

  return (
    conn.id.toLowerCase().includes(searchText) ||
    sourceText.includes(searchText) ||
    destText.includes(searchText) ||
    (conn.rule && conn.rule.toLowerCase().includes(searchText)) ||
    (conn.metadata.processPath?.toLowerCase() || '').includes(searchText)
  )
}

// 刷新连接列表
const refreshConnections = async () => {
  loading.value = true
  try {
    // 这里实际上不需要做什么，因为connectionStore中的connections已经通过WebSocket自动更新
    // 但我们仍然提供刷新按钮以便于用户手动刷新界面
    message.success(t('connections.refreshSuccess'))
  } catch (error) {
    console.error(t('connections.refreshError'), error)
    message.error(t('connections.refreshError', { error: String(error) }))
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  // 当组件挂载时，确保连接数据已经初始化
  if (!connections.value.length) {
    // 设置连接监听器
    await connectionStore.setupEventListeners()
    refreshConnections()
  }
})

// 组件卸载时清理连接监听器
onUnmounted(() => {
  connectionStore.cleanupEventListeners()
})
</script>

<style scoped>
.connections-page {
  animation: fadeIn 0.4s ease both;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.filter-panel {
  border-radius: 28px;
}

.filter-content {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.filter-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1;
}

.filter-select {
  min-width: 200px;
  flex: 1;
}

.active-filters {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.connections-card {
  border-radius: 32px;
}

.connections-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
}

.connection-item {
  position: relative;
  border-radius: 24px;
  padding: 20px;
  background: rgba(15, 23, 42, 0.02);
  border: 1px solid var(--panel-border);
  overflow: hidden;
  transition: transform 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease;
}

.connection-item:hover {
  transform: translateY(-3px);
  border-color: rgba(91, 76, 253, 0.4);
  box-shadow: 0 25px 40px rgba(15, 23, 42, 0.15);
}

.connection-highlight {
  border-color: rgba(91, 76, 253, 0.45);
}

.connection-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}

.connection-time {
  font-size: 13px;
  color: var(--text-muted);
}

.connection-details {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.connection-row {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}

.detail-item {
  flex: 1;
  min-width: 220px;
}

.detail-label {
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.detail-value {
  font-size: 14px;
  color: var(--text-primary);
  font-weight: 600;
  word-break: break-word;
}

.traffic-value {
  display: flex;
  gap: 12px;
  font-weight: 600;
}

.traffic-value .upload {
  color: #ef4444;
}

.traffic-value .download {
  color: #10b981;
}

.connection-indicator {
  position: absolute;
  inset: 0;
  border-radius: 24px;
  border: 1px solid transparent;
  pointer-events: none;
}

.connection-highlight .connection-indicator {
  border-color: rgba(91, 76, 253, 0.4);
  box-shadow: inset 0 0 20px rgba(91, 76, 253, 0.25);
}

.empty-state {
  margin-top: 12px;
}

@media (max-width: 768px) {
  .connections-grid {
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  }

  .filter-select {
    min-width: 160px;
  }
}
</style>
