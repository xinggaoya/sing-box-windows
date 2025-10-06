<template>
  <div class="ultra-connections">
    <!-- 紧凑工具栏 -->
    <div class="connections-toolbar">
      <div class="toolbar-left">
        <div class="toolbar-icon">
          <n-icon size="16">
            <LinkOutline />
          </n-icon>
        </div>
        <div class="toolbar-info">
          <span class="toolbar-title">{{ t('connections.title') }}</span>
          <span class="toolbar-stats">{{ connections.length }} {{ t('connections.totalCount', { count: connections.length }) }}</span>
        </div>
      </div>

      <div class="toolbar-right">
        <n-button
          @click="refreshConnections"
          :loading="loading"
          type="primary"
          size="small"
          class="refresh-btn"
        >
          <template #icon>
            <n-icon size="12"><RefreshOutline /></n-icon>
          </template>
          {{ t('common.refresh') }}
        </n-button>
      </div>
    </div>

    <!-- 统计面板 -->
    <div class="stats-panel">
      <div class="stat-orb active-orb">
        <div class="orb-icon">
          <n-icon size="14"><LinkOutline /></n-icon>
        </div>
        <div class="orb-content">
          <div class="orb-value">
            <n-number-animation ref="activeCountRef" :from="0" :to="connections.length" />
          </div>
          <div class="orb-label">{{ t('connections.activeConnections') }}</div>
        </div>
      </div>

      <div class="stat-orb upload-orb">
        <div class="orb-icon">
          <n-icon size="14"><ArrowUpOutline /></n-icon>
        </div>
        <div class="orb-content">
          <div class="orb-value">{{ formatBytes(connectionsTotal.upload) }}</div>
          <div class="orb-label">{{ t('home.traffic.uploadTotal') }}</div>
        </div>
      </div>

      <div class="stat-orb download-orb">
        <div class="orb-icon">
          <n-icon size="14"><ArrowDownOutline /></n-icon>
        </div>
        <div class="orb-content">
          <div class="orb-value">{{ formatBytes(connectionsTotal.download) }}</div>
          <div class="orb-label">{{ t('home.traffic.downloadTotal') }}</div>
        </div>
      </div>

      <div class="stat-orb filtered-orb">
        <div class="orb-icon">
          <n-icon size="14"><HardwareChipOutline /></n-icon>
        </div>
        <div class="orb-content">
          <div class="orb-value">{{ filteredConnections.length }}</div>
          <div class="orb-label">{{ t('connections.matchedConnections') }}</div>
        </div>
      </div>
    </div>

    <!-- 连接内容区域 -->
    <div class="connections-content">
      <!-- 搜索筛选区域 -->
      <div class="search-section">
        <div class="search-input-group">
          <n-input
            v-model:value="searchQuery"
            :placeholder="t('connections.searchPlaceholder')"
            clearable
            size="small"
            class="search-input"
          >
            <template #prefix>
              <n-icon size="14">
                <SearchOutline />
              </n-icon>
            </template>
          </n-input>

          <div class="filter-selects">
            <n-select
              v-model:value="networkFilter"
              :options="networkOptions"
              :placeholder="t('connections.networkTypeFilter')"
              clearable
              size="small"
              class="filter-select"
            />
            <n-select
              v-model:value="ruleFilter"
              :options="ruleOptions"
              :placeholder="t('connections.ruleFilter')"
              clearable
              size="small"
              class="filter-select"
            />
          </div>
        </div>

        <div class="filter-tags" v-if="searchQuery || networkFilter || ruleFilter">
          <n-tag v-if="searchQuery" size="tiny" round class="filter-tag">
            {{ t('common.search') }}: {{ searchQuery }}
          </n-tag>
          <n-tag v-if="networkFilter" size="tiny" round class="filter-tag">
            {{ t('connections.networkTypeFilter') }}: {{ networkFilter }}
          </n-tag>
          <n-tag v-if="ruleFilter" size="tiny" round class="filter-tag">
            {{ t('connections.ruleFilter') }}: {{ ruleFilter }}
          </n-tag>
        </div>
      </div>

      <!-- 连接列表 -->
      <div class="connections-list">
        <n-spin :show="loading">
          <div v-if="filteredConnections.length > 0" class="connections-grid">
            <div
              v-for="(conn, index) in filteredConnections"
              :key="conn.id"
              class="connection-item"
              :class="{ 'connection-highlight': isConnectionHighlighted(conn) }"
            >
              <!-- 连接ID -->
              <div class="connection-id">
                <div class="id-badge">
                  {{ getConnectionShortId(conn.id) }}
                </div>
              </div>

              <!-- 时间 -->
              <div class="connection-time">
                <div class="time-text">
                  {{ formatConnectionTime(conn.start) }}
                </div>
              </div>

              <!-- 网络类型 -->
              <div class="connection-network">
                <div class="network-badge" :class="getNetworkClass(conn.metadata.network)">
                  {{ conn.metadata.network?.toUpperCase() }}
                </div>
              </div>

              <!-- 源地址 -->
              <div class="connection-source">
                <div class="source-text" :title="getSourceText(conn)">
                  {{ getSourceText(conn) }}
                </div>
              </div>

              <!-- 目标地址 -->
              <div class="connection-destination">
                <div class="dest-text" :title="getDestinationText(conn)">
                  {{ getDestinationText(conn) }}
                </div>
              </div>

              <!-- 规则 -->
              <div class="connection-rule">
                <div class="rule-badge" :class="getRuleClass(conn.rule)">
                  {{ conn.rule }}
                </div>
              </div>

              <!-- 流量 -->
              <div class="connection-traffic">
                <div class="traffic-info">
                  <div class="traffic-upload">↑{{ formatBytes(conn.upload) }}</div>
                  <div class="traffic-download">↓{{ formatBytes(conn.download) }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-else class="empty-state">
            <div class="empty-icon">
              <n-icon size="32">
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
              size="medium"
              class="empty-btn"
            >
              <template #icon>
                <n-icon size="14"><RefreshOutline /></n-icon>
              </template>
              {{ t('connections.refreshConnections') }}
            </n-button>
          </div>
        </n-spin>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, h, computed, watch } from 'vue'
import { useMessage, NTag, DataTableColumns, NSpace, NTooltip, NText, SelectOption } from 'naive-ui'
import {
  RefreshOutline,
  SearchOutline,
  LinkOutline,
  ArrowUpOutline,
  ArrowDownOutline,
  HardwareChipOutline,
} from '@vicons/ionicons5'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useI18n } from 'vue-i18n'
import { useThemeStore } from '@/stores/app/ThemeStore'

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

// 格式化时间
const formatTime = (timeString: string) => {
  try {
    const date = new Date(timeString)
    return date.toLocaleString()
  } catch (e) {
    return timeString
  }
}

// 定义表格列
const columns: DataTableColumns<Connection> = [
  {
    title: t('connections.unknown'),
    key: 'id',
    width: 100,
    ellipsis: {
      tooltip: true,
    },
    render(row: Connection) {
      // 高亮搜索关键字
      if (searchQuery.value && row.id.toLowerCase().includes(searchQuery.value.toLowerCase())) {
        const index = row.id.toLowerCase().indexOf(searchQuery.value.toLowerCase())
        const beforeMatch = row.id.substring(0, index)
        const match = row.id.substring(index, index + searchQuery.value.length)
        const afterMatch = row.id.substring(index + searchQuery.value.length)

        return h('div', {}, [
          beforeMatch,
          h(
            'span',
            { style: { backgroundColor: 'rgba(var(--primary-color), 0.1)', fontWeight: 'bold' } },
            match,
          ),
          afterMatch,
        ])
      }
      return row.id
    },
  },
  {
    title: t('connections.startTime'),
    key: 'start',
    width: 160,
    render(row: Connection) {
      return formatTime(row.start)
    },
  },
  {
    title: t('connections.networkType'),
    key: 'network',
    width: 120,
    render(row: Connection) {
      const { network, type } = row.metadata
      return h(
        NSpace,
        { vertical: true, size: 'small' },
        {
          default: () => [
            h(
              NTag,
              {
                type: network === 'tcp' ? 'info' : 'warning',
                size: 'small',
                bordered: false,
              },
              {
                default: () =>
                  typeof network === 'string'
                    ? network.toUpperCase()
                    : String(network).toUpperCase(),
              },
            ),
            h(
              NTag,
              {
                type: 'default',
                size: 'small',
                bordered: false,
              },
              { default: () => (typeof type === 'string' ? type : String(type)) },
            ),
          ],
        },
      )
    },
  },
  {
    title: t('connections.source'),
    key: 'source',
    width: 200,
    render(row: Connection) {
      const { sourceIP, sourcePort } = row.metadata
      const sourceText = `${sourceIP}:${sourcePort}`

      // 高亮搜索关键字
      if (searchQuery.value && sourceText.toLowerCase().includes(searchQuery.value.toLowerCase())) {
        const index = sourceText.toLowerCase().indexOf(searchQuery.value.toLowerCase())
        const beforeMatch = sourceText.substring(0, index)
        const match = sourceText.substring(index, index + searchQuery.value.length)
        const afterMatch = sourceText.substring(index + searchQuery.value.length)

        return h(
          NTooltip,
          {},
          {
            trigger: () =>
              h('div', {}, [
                beforeMatch,
                h(
                  'span',
                  {
                    style: {
                      backgroundColor: 'rgba(var(--primary-color), 0.1)',
                      fontWeight: 'bold',
                    },
                  },
                  match,
                ),
                afterMatch,
              ]),
            default: () =>
              h('div', {}, [
                h('div', {}, `${t('connections.ip')}: ${sourceIP}`),
                h('div', {}, `${t('connections.port')}: ${sourcePort}`),
              ]),
          },
        )
      }

      return h(
        NTooltip,
        {},
        {
          trigger: () => sourceText,
          default: () =>
            h('div', {}, [
              h('div', {}, `${t('connections.ip')}: ${sourceIP}`),
              h('div', {}, `${t('connections.port')}: ${sourcePort}`),
            ]),
        },
      )
    },
  },
  {
    title: t('connections.destination'),
    key: 'destination',
    width: 200,
    render(row: Connection) {
      const { destinationIP, destinationPort, host } = row.metadata
      const destText = host || `${destinationIP}:${destinationPort}`

      // 高亮搜索关键字
      if (searchQuery.value && destText.toLowerCase().includes(searchQuery.value.toLowerCase())) {
        const index = destText.toLowerCase().indexOf(searchQuery.value.toLowerCase())
        const beforeMatch = destText.substring(0, index)
        const match = destText.substring(index, index + searchQuery.value.length)
        const afterMatch = destText.substring(index + searchQuery.value.length)

        return h(
          NTooltip,
          {},
          {
            trigger: () =>
              h('div', {}, [
                beforeMatch,
                h(
                  'span',
                  {
                    style: {
                      backgroundColor: 'rgba(var(--primary-color), 0.1)',
                      fontWeight: 'bold',
                    },
                  },
                  match,
                ),
                afterMatch,
              ]),
            default: () =>
              h('div', {}, [
                host ? h('div', {}, `${t('connections.host')}: ${host}`) : null,
                h('div', {}, `${t('connections.ip')}: ${destinationIP}`),
                h('div', {}, `${t('connections.port')}: ${destinationPort}`),
              ]),
          },
        )
      }

      return h(
        NTooltip,
        {},
        {
          trigger: () => destText,
          default: () =>
            h('div', {}, [
              host ? h('div', {}, `${t('connections.host')}: ${host}`) : null,
              h('div', {}, `${t('connections.ip')}: ${destinationIP}`),
              h('div', {}, `${t('connections.port')}: ${destinationPort}`),
            ]),
        },
      )
    },
  },
  {
    title: t('connections.rule'),
    key: 'rule',
    width: 160,
    render(row: Connection) {
      // 高亮搜索关键字
      if (
        searchQuery.value &&
        row.rule &&
        String(row.rule).toLowerCase().includes(searchQuery.value.toLowerCase())
      ) {
        const index = String(row.rule).toLowerCase().indexOf(searchQuery.value.toLowerCase())
        const beforeMatch = row.rule.substring(0, index)
        const match = row.rule.substring(index, index + searchQuery.value.length)
        const afterMatch = row.rule.substring(index + searchQuery.value.length)

        return h(
          NSpace,
          { vertical: true, size: 'small' },
          {
            default: () => [
              h(
                NTag,
                {
                  type: 'success',
                  size: 'small',
                  bordered: false,
                },
                {
                  default: () =>
                    h('div', {}, [
                      beforeMatch,
                      h(
                        'span',
                        {
                          style: {
                            backgroundColor: 'rgba(var(--primary-color), 0.1)',
                            fontWeight: 'bold',
                          },
                        },
                        match,
                      ),
                      afterMatch,
                    ]),
                },
              ),
              row.rulePayload
                ? h(NText, { depth: 3, size: 'small' }, { default: () => row.rulePayload })
                : null,
            ],
          },
        )
      }

      return h(
        NSpace,
        { vertical: true, size: 'small' },
        {
          default: () => [
            h(
              NTag,
              {
                type: 'success',
                size: 'small',
                bordered: false,
              },
              { default: () => row.rule },
            ),
            row.rulePayload
              ? h(NText, { depth: 3, size: 'small' }, { default: () => row.rulePayload })
              : null,
          ],
        },
      )
    },
  },
  {
    title: t('connections.process'),
    key: 'process',
    ellipsis: {
      tooltip: true,
    },
    render(row: Connection) {
      const processPath = row.metadata.processPath || t('connections.unknown')

      // 高亮搜索关键字
      if (
        searchQuery.value &&
        processPath.toLowerCase().includes(searchQuery.value.toLowerCase())
      ) {
        const index = processPath.toLowerCase().indexOf(searchQuery.value.toLowerCase())
        const beforeMatch = processPath.substring(0, index)
        const match = processPath.substring(index, index + searchQuery.value.length)
        const afterMatch = processPath.substring(index + searchQuery.value.length)

        return h('div', {}, [
          beforeMatch,
          h(
            'span',
            { style: { backgroundColor: 'rgba(var(--primary-color), 0.1)', fontWeight: 'bold' } },
            match,
          ),
          afterMatch,
        ])
      }

      return processPath
    },
  },
  {
    title: t('connections.traffic'),
    key: 'traffic',
    width: 160,
    render(row: Connection) {
      return h(
        NSpace,
        { vertical: true, size: 'small' },
        {
          default: () => [
            h(
              NSpace,
              { align: 'center', size: 'small' },
              {
                default: () => [
                  h(
                    NTag,
                    { type: 'error', size: 'small', bordered: false },
                    { default: () => '↑' },
                  ),
                  h(NText, {}, { default: () => formatBytes(row.upload) }),
                ],
              },
            ),
            h(
              NSpace,
              { align: 'center', size: 'small' },
              {
                default: () => [
                  h(NTag, { type: 'info', size: 'small', bordered: false }, { default: () => '↓' }),
                  h(NText, {}, { default: () => formatBytes(row.download) }),
                ],
              },
            ),
          ],
        },
      )
    },
  },
]

// 分页设置
const pagination = {
  pageSize: 15,
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

const getRuleClass = (rule: string): string => {
  if (!rule) return 'rule-default'
  if (rule.includes('direct')) return 'rule-direct'
  if (rule.includes('proxy')) return 'rule-proxy'
  if (rule.includes('reject')) return 'rule-reject'
  return 'rule-normal'
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
.ultra-connections {
  padding: 16px;
  background: var(--n-color-embedded);
  min-height: calc(100vh - 36px);
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: slideFadeIn 0.4s ease-out;
}

/* 紧凑工具栏 */
.connections-toolbar {
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toolbar-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 12px rgba(0, 180, 42, 0.3);
}

.toolbar-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toolbar-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0;
}

.toolbar-stats {
  font-size: 0.75rem;
  color: var(--n-text-color-3);
  margin: 0;
}

.toolbar-right {
  display: flex;
  gap: 8px;
}

.refresh-btn {
  height: 32px;
  padding: 0 12px;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.refresh-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 180, 42, 0.3);
}

/* 统计面板 */
.stats-panel {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
}

.stat-orb {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  border-radius: 8px;
  transition: all 0.2s ease;
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.stat-orb:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.stat-orb::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  border-radius: 8px 8px 0 0;
}

.active-orb::before {
  background: linear-gradient(90deg, #00b42a 0%, #009a1a 100%);
}

.upload-orb::before {
  background: linear-gradient(90deg, #f53f3f 0%, #cb2a2a 100%);
}

.download-orb::before {
  background: linear-gradient(90deg, #4080ff 0%, #2266dd 100%);
}

.filtered-orb::before {
  background: linear-gradient(90deg, #ff9500 0%, #ff6200 100%);
}

.orb-icon {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 500;
}

.active-orb .orb-icon {
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
}

.upload-orb .orb-icon {
  background: linear-gradient(135deg, #f53f3f 0%, #cb2a2a 100%);
}

.download-orb .orb-icon {
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 100%);
}

.filtered-orb .orb-icon {
  background: linear-gradient(135deg, #ff9500 0%, #ff6200 100%);
}

.orb-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.orb-value {
  font-size: 1rem;
  font-weight: 700;
  color: var(--n-text-color-1);
  line-height: 1.2;
}

.orb-label {
  font-size: 0.7rem;
  color: var(--n-text-color-3);
  font-weight: 500;
}

/* 连接内容区域 */
.connections-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 0;
}

/* 搜索筛选区域 */
.search-section {
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
}

.search-input-group {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1;
  min-width: 280px;
}

.search-input :deep(.n-input) {
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
}

.search-input :deep(.n-input:hover) {
  border-color: #00b42a;
}

.search-input :deep(.n-input.n-input--focus) {
  border-color: #00b42a;
  box-shadow: 0 0 0 2px rgba(0, 180, 42, 0.1);
}

.filter-selects {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-select {
  min-width: 140px;
}

.filter-select :deep(.n-base-selection) {
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
}

.filter-select :deep(.n-base-selection:hover) {
  border-color: #00b42a;
}

.filter-select :deep(.n-base-selection.n-base-selection--focus) {
  border-color: #00b42a;
  box-shadow: 0 0 0 2px rgba(0, 180, 42, 0.1);
}

.filter-tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-tag {
  font-size: 0.75rem;
  font-weight: 500;
}

/* 连接列表 */
.connections-list {
  flex: 1;
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  min-height: 0;
}

.connections-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.connection-item {
  display: grid;
  grid-template-columns: 60px 50px 60px 120px 140px 100px 100px;
  gap: 8px;
  align-items: center;
  padding: 10px 12px;
  background: var(--n-color-embedded);
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.connection-item:hover {
  background: var(--n-color-embedded-modal);
  border-color: #00b42a;
  transform: translateX(2px);
  box-shadow: 0 2px 8px rgba(0, 180, 42, 0.1);
}

.connection-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: transparent;
  transition: background 0.2s ease;
}

.connection-item:hover::before {
  background: #00b42a;
}

.connection-highlight {
  background: rgba(0, 180, 42, 0.05);
  border-color: rgba(0, 180, 42, 0.2);
}

.connection-highlight::before {
  background: #00b42a;
}

.connection-id {
  display: flex;
  align-items: center;
}

.id-badge {
  padding: 3px 6px;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 500;
  background: rgba(107, 114, 128, 0.1);
  color: var(--n-text-color-2);
  text-align: center;
  white-space: nowrap;
}

.connection-time {
  display: flex;
  align-items: center;
}

.time-text {
  font-size: 0.75rem;
  color: var(--n-text-color-2);
  font-weight: 500;
}

.connection-network {
  display: flex;
  align-items: center;
}

.network-badge {
  padding: 3px 6px;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 500;
  text-align: center;
  white-space: nowrap;
}

.network-tcp {
  background: rgba(64, 128, 255, 0.1);
  color: #4080ff;
  border: 1px solid rgba(64, 128, 255, 0.2);
}

.network-udp {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.2);
}

.network-other {
  background: rgba(107, 114, 128, 0.1);
  color: #6b7280;
  border: 1px solid rgba(107, 114, 128, 0.2);
}

.connection-source,
.connection-destination {
  flex: 1;
  min-width: 0;
}

.source-text,
.dest-text {
  font-size: 0.75rem;
  color: var(--n-text-color-1);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.connection-rule {
  display: flex;
  align-items: center;
}

.rule-badge {
  padding: 3px 6px;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 500;
  text-align: center;
  white-space: nowrap;
}

.rule-direct {
  background: rgba(16, 185, 129, 0.1);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.rule-proxy {
  background: rgba(64, 128, 255, 0.1);
  color: #4080ff;
  border: 1px solid rgba(64, 128, 255, 0.2);
}

.rule-reject {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.rule-normal {
  background: rgba(107, 114, 128, 0.1);
  color: #6b7280;
  border: 1px solid rgba(107, 114, 128, 0.2);
}

.rule-default {
  background: rgba(107, 114, 128, 0.1);
  color: #6b7280;
  border: 1px solid rgba(107, 114, 128, 0.2);
}

.connection-traffic {
  display: flex;
  align-items: center;
}

.traffic-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.traffic-upload,
.traffic-download {
  font-size: 0.7rem;
  font-weight: 500;
  line-height: 1.2;
}

.traffic-upload {
  color: #f53f3f;
}

.traffic-download {
  color: #4080ff;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  padding: 40px 20px;
  text-align: center;
}

.empty-icon {
  color: var(--n-text-color-disabled);
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0 0 8px 0;
}

.empty-desc {
  font-size: 0.875rem;
  color: var(--n-text-color-3);
  margin: 0 0 20px 0;
  line-height: 1.5;
  max-width: 300px;
}

.empty-btn {
  height: 36px;
  padding: 0 16px;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.empty-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 180, 42, 0.3);
}

/* 动画效果 */
@keyframes slideFadeIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 响应式设计 */
@media (max-width: 1024px) {
  .stats-panel {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .ultra-connections {
    padding: 12px;
    gap: 12px;
  }

  .connections-toolbar {
    padding: 10px 12px;
  }

  .toolbar-icon {
    width: 28px;
    height: 28px;
  }

  .toolbar-title {
    font-size: 0.875rem;
  }

  .toolbar-stats {
    font-size: 0.7rem;
  }

  .stats-panel {
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    padding: 12px;
  }

  .stat-orb {
    padding: 8px;
    flex-direction: column;
    text-align: center;
    gap: 4px;
  }

  .orb-icon {
    width: 24px;
    height: 24px;
  }

  .orb-value {
    font-size: 0.875rem;
  }

  .orb-label {
    font-size: 0.65rem;
  }

  .search-input-group {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .search-input {
    min-width: unset;
  }

  .filter-selects {
    justify-content: space-between;
  }

  .filter-select {
    flex: 1;
    min-width: 120px;
  }

  .connection-item {
    grid-template-columns: 50px 40px 50px 1fr 80px 80px;
    gap: 6px;
    padding: 8px 10px;
  }

  .connection-time {
    display: none;
  }

  .source-text,
  .dest-text {
    font-size: 0.7rem;
  }
}

@media (max-width: 480px) {
  .ultra-connections {
    padding: 8px;
    gap: 8px;
  }

  .connections-toolbar {
    padding: 8px 10px;
  }

  .toolbar-left {
    gap: 8px;
  }

  .toolbar-icon {
    width: 24px;
    height: 24px;
  }

  .toolbar-title {
    font-size: 0.8rem;
  }

  .search-section {
    padding: 12px;
  }

  .connections-list {
    padding: 12px;
  }

  .stats-panel {
    grid-template-columns: repeat(2, 1fr);
  }

  .connection-item {
    grid-template-columns: 1fr;
    gap: 4px;
    padding: 8px 10px;
  }

  .connection-id,
  .connection-network,
  .connection-rule,
  .connection-traffic {
    display: none;
  }

  .source-text,
  .dest-text {
    font-size: 0.8rem;
    white-space: normal;
    overflow: visible;
    text-overflow: unset;
    line-height: 1.4;
  }

  .empty-state {
    padding: 32px 16px;
    min-height: 250px;
  }

  .empty-title {
    font-size: 1rem;
  }

  .empty-desc {
    font-size: 0.8rem;
  }
}

/* Naive UI 组件优化 */
:deep(.n-spin-container) {
  min-height: 200px;
}

:deep(.n-input__input-el) {
  font-size: 0.875rem !important;
}

:deep(.n-base-selection-label) {
  font-size: 0.875rem !important;
}

:deep(.n-button__content) {
  font-size: 0.875rem !important;
}
</style>
