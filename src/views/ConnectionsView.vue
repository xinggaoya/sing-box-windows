<template>
  <div class="connections-page">
    <!-- 页面标题和统计 -->
    <div class="page-header">
      <div class="header-content">
        <div class="header-left">
          <div class="header-icon">
            <n-icon size="20">
              <LinkOutline />
            </n-icon>
          </div>
          <div class="header-info">
            <h1 class="page-title">{{ t('connections.title') }}</h1>
            <p class="page-subtitle">{{ t('connections.subtitle') }}</p>
          </div>
        </div>
        <div class="header-actions">
          <n-button
            @click="refreshConnections"
            :loading="loading"
            type="primary"
            size="medium"
            class="refresh-btn"
          >
            <template #icon>
              <n-icon size="16">
                <RefreshOutline />
              </n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
        </div>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <n-card class="stat-card active-card" :bordered="false">
        <div class="stat-content">
          <div class="stat-icon">
            <n-icon size="24">
              <LinkOutline />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">
              <n-number-animation ref="activeCountRef" :from="0" :to="connections.length" />
            </div>
            <div class="stat-label">{{ t('connections.activeConnections') }}</div>
          </div>
        </div>
      </n-card>

      <n-card class="stat-card upload-card" :bordered="false">
        <div class="stat-content">
          <div class="stat-icon">
            <n-icon size="24">
              <ArrowUpOutline />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ formatBytes(connectionsTotal.upload) }}</div>
            <div class="stat-label">{{ t('home.traffic.uploadTotal') }}</div>
          </div>
        </div>
      </n-card>

      <n-card class="stat-card download-card" :bordered="false">
        <div class="stat-content">
          <div class="stat-icon">
            <n-icon size="24">
              <ArrowDownOutline />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ formatBytes(connectionsTotal.download) }}</div>
            <div class="stat-label">{{ t('home.traffic.downloadTotal') }}</div>
          </div>
        </div>
      </n-card>

      <n-card class="stat-card filtered-card" :bordered="false">
        <div class="stat-content">
          <div class="stat-icon">
            <n-icon size="24">
              <FilterOutline />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ filteredConnections.length }}</div>
            <div class="stat-label">{{ t('connections.matchedConnections') }}</div>
          </div>
        </div>
      </n-card>
    </div>

    <!-- 搜索和筛选 -->
    <n-card class="filter-card" :bordered="false">
      <div class="filter-content">
        <div class="filter-row">
          <n-input
            v-model:value="searchQuery"
            :placeholder="t('connections.searchPlaceholder')"
            clearable
            size="medium"
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
            size="medium"
            class="filter-select"
          />
          <n-select
            v-model:value="ruleFilter"
            :options="ruleOptions"
            :placeholder="t('connections.ruleFilter')"
            clearable
            size="medium"
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

    <!-- 连接列表 -->
    <n-card class="connections-card" :bordered="false">
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
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

defineOptions({
  name: 'ConnectionsView'
})

const message = useMessage()
const loading = ref(false)
const connectionStore = useConnectionStore()
const { t } = useI18n()
const activeCountRef = ref(null)
const themeStore = useThemeStore()

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

// 监听连接变化以更新动画
watch(
  () => connections.value.length,
  (newVal, oldVal) => {
    if (activeCountRef.value && newVal !== oldVal) {
      // @ts-expect-error - 忽略类型错误，因为 NNumberAnimation 组件确实有 play 方法
      activeCountRef.value.play()
    }
  },
)

// 格式化字节大小的函数
const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return (bytes / Math.pow(k, i)).toFixed(2) + ' ' + sizes[i]
}

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
      return Math.floor(diff / 1000) + 's'
    } else if (diff < 3600000) {
      return Math.floor(diff / 60000) + 'm'
    } else if (diff < 86400000) {
      return Math.floor(diff / 3600000) + 'h'
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
  if (!rule) return 'default'
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
    message.error(`${t('connections.refreshError')}: ${error}`)
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
  connectionStore.cleanupListeners()
})
</script>

<style scoped>
.connections-page {
  padding: 16px;
  min-height: calc(100vh - 48px);
  background: v-bind('themeStore.isDark ? "#18181b" : "#f8fafc"');
}

/* 页面标题 */
.page-header {
  margin-bottom: 24px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border-radius: 16px;
  padding: 24px 28px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0;
  letter-spacing: -0.02em;
}

.page-subtitle {
  font-size: 14px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0;
  font-weight: 400;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.refresh-btn {
  height: 42px;
  padding: 0 16px;
  font-weight: 600;
  border-radius: 10px;
  transition: all 0.2s ease;
}

.refresh-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

/* 统计卡片 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 20px;
  margin-bottom: 24px;
}

.stat-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
  transition: all 0.3s ease;
  overflow: hidden;
  position: relative;
}

.stat-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
}

.active-card::before {
  background: linear-gradient(90deg, #10b981 0%, #059669 100%);
}

.upload-card::before {
  background: linear-gradient(90deg, #ef4444 0%, #dc2626 100%);
}

.download-card::before {
  background: linear-gradient(90deg, #3b82f6 0%, #2563eb 100%);
}

.filtered-card::before {
  background: linear-gradient(90deg, #f59e0b 0%, #d97706 100%);
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.1)"');
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.active-card .stat-icon {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
}

.upload-card .stat-icon {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
}

.download-card .stat-icon {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
}

.filtered-card .stat-icon {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  line-height: 1.2;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
}

/* 筛选卡片 */
.filter-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
  margin-bottom: 24px;
}

.filter-content {
  padding: 8px;
}

.filter-row {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
}

.filter-row:last-child {
  margin-bottom: 0;
}

.search-input {
  flex: 1;
}

.filter-select {
  flex: 1;
  min-width: 200px;
}

.active-filters {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  padding-top: 8px;
  border-top: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

/* 连接卡片 */
.connections-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
}

.connections-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.connection-item {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.02)" : "rgba(0, 0, 0, 0.02)"');
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  border-radius: 12px;
  padding: 16px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.connection-item:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.04)" : "rgba(0, 0, 0, 0.04)"');
  border-color: v-bind('themeStore.isDark ? "rgba(91, 76, 253, 0.3)" : "rgba(91, 76, 253, 0.2)"');
  transform: translateX(4px);
}

.connection-highlight {
  background: rgba(91, 76, 253, 0.05);
  border-color: rgba(91, 76, 253, 0.3);
}

.connection-indicator {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
}

.network-tcp {
  background: #3b82f6;
}

.network-udp {
  background: #f59e0b;
}

.network-other {
  background: #6b7280;
}

.connection-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.connection-time {
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
}

.connection-details {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.connection-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-label {
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.detail-value {
  font-size: 13px;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.traffic-value {
  display: flex;
  gap: 8px;
}

.traffic-value .upload {
  color: #ef4444;
  font-weight: 600;
}

.traffic-value .download {
  color: #3b82f6;
  font-weight: 600;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  padding: 40px 16px;
  text-align: center;
}

.empty-icon {
  color: v-bind('themeStore.isDark ? "#4b5563" : "#9ca3af"');
  margin-bottom: 12px;
  opacity: 0.6;
}

.empty-title {
  font-size: 20px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0 0 12px 0;
}

.empty-desc {
  font-size: 14px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0 0 24px 0;
  line-height: 1.5;
  max-width: 400px;
}

.empty-btn {
  height: 42px;
  padding: 0 24px;
  font-weight: 600;
  border-radius: 10px;
  transition: all 0.2s ease;
}

.empty-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .connections-page {
    padding: 16px;
  }

  .header-content {
    flex-direction: column;
    gap: 20px;
    padding: 16px;
  }

  .header-left {
    width: 100%;
  }

  .header-actions {
    width: 100%;
    justify-content: center;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .stat-content {
    padding: 16px;
    gap: 12px;
  }

  .stat-icon {
    width: 40px;
    height: 40px;
  }

  .stat-value {
    font-size: 20px;
  }

  .filter-row {
    flex-direction: column;
    gap: 12px;
  }

  .connection-row {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .detail-value {
    white-space: normal;
    line-height: 1.4;
  }
}

@media (max-width: 480px) {
  .connections-page {
    padding: 12px;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .header-content {
    padding: 16px;
  }

  .page-title {
    font-size: 20px;
  }

  .page-subtitle {
    font-size: 13px;
  }

  .connection-item {
    padding: 12px;
  }

  .empty-state {
    padding: 40px 16px;
    min-height: 300px;
  }

  .empty-title {
    font-size: 18px;
  }

  .empty-desc {
    font-size: 13px;
  }
}

/* Naive UI 组件优化 */
:deep(.n-spin-container) {
  min-height: 200px;
}

:deep(.n-input) {
  border-radius: 10px;
}

:deep(.n-base-selection) {
  border-radius: 10px;
}

:deep(.n-tag) {
  border-radius: 6px;
  font-weight: 500;
}
</style>