<template>
  <div class="connections-view">
    <!-- 英雄式页面头部 -->
    <div class="hero-header">
      <div class="hero-content">
        <div class="hero-icon">
          <n-icon size="48">
            <link-outline />
          </n-icon>
        </div>
        <div class="hero-text">
          <h1 class="hero-title">{{ t('connections.title') }}</h1>
          <p class="hero-subtitle">{{ t('connections.subtitle') }}</p>
        </div>
        <div class="hero-action">
          <n-button
            type="primary"
            @click="refreshConnections"
            :loading="loading"
            size="large"
            round
            class="hero-btn"
          >
            <template #icon>
              <n-icon><refresh-outline /></n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
        </div>
      </div>
    </div>

    <!-- 统计卡片网格 -->
    <div class="stats-grid">
      <div class="stat-card active-connections">
        <div class="stat-icon">
          <n-icon size="24"><link-outline /></n-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">
            <n-number-animation ref="activeCountRef" :from="0" :to="connections.length" />
          </div>
          <div class="stat-label">{{ t('connections.activeConnections') }}</div>
        </div>
      </div>

      <div class="stat-card upload-stats">
        <div class="stat-icon">
          <n-icon size="24"><arrow-up-outline /></n-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatBytes(connectionsTotal.upload) }}</div>
          <div class="stat-label">{{ t('connections.uploadTotal') }}</div>
        </div>
      </div>

      <div class="stat-card download-stats">
        <div class="stat-icon">
          <n-icon size="24"><arrow-down-outline /></n-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatBytes(connectionsTotal.download) }}</div>
          <div class="stat-label">{{ t('connections.downloadTotal') }}</div>
        </div>
      </div>

      <div class="stat-card memory-stats">
        <div class="stat-icon">
          <n-icon size="24"><hardware-chip-outline /></n-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ filteredConnections.length }}</div>
          <div class="stat-label">{{ t('connections.matchedConnections') }}</div>
        </div>
      </div>
    </div>

    <!-- 搜索筛选卡片 -->
    <div class="filter-card">
      <div class="filter-header">
        <h3 class="filter-title">{{ t('connections.searchAndFilter') }}</h3>
        <div class="filter-stats">
          <n-tag type="info" size="small" round>
            {{ t('connections.totalCount', { count: connections.length }) }}
          </n-tag>
          <n-tag
            v-if="searchQuery || networkFilter || ruleFilter"
            type="success"
            size="small"
            round
          >
            {{ t('connections.matchingCount', { count: filteredConnections.length }) }}
          </n-tag>
        </div>
      </div>

      <div class="filter-controls">
        <n-input
          v-model:value="searchQuery"
          :placeholder="t('connections.searchPlaceholder')"
          clearable
          size="large"
          class="search-input"
        >
          <template #prefix>
            <n-icon><search-outline /></n-icon>
          </template>
        </n-input>

        <div class="filter-selects">
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
      </div>
    </div>

    <!-- 连接表格卡片 -->
    <div class="table-card">
      <n-spin :show="loading">
        <div v-if="filteredConnections.length > 0" class="table-container">
          <n-data-table
            :columns="columns"
            :data="filteredConnections"
            :pagination="pagination"
            :bordered="false"
            :max-height="500"
            striped
            class="connections-table"
          />
        </div>
        <div v-else class="empty-state">
          <div class="empty-icon">
            <n-icon size="64">
              <link-outline />
            </n-icon>
          </div>
          <h3 class="empty-title">
            {{
              searchQuery || networkFilter || ruleFilter
                ? t('connections.noMatchingConnections2')
                : t('connections.noActiveConnections')
            }}
          </h3>
          <p class="empty-description">
            {{
              searchQuery || networkFilter || ruleFilter
                ? t('connections.adjustSearchOrFilters')
                : t('connections.noActiveNetworkConnections')
            }}
          </p>
          <n-button
            v-if="!searchQuery && !networkFilter && !ruleFilter"
            @click="refreshConnections"
            type="primary"
            size="large"
            round
            class="empty-action"
          >
            <template #icon>
              <n-icon><refresh-outline /></n-icon>
            </template>
            {{ t('connections.refreshConnections') }}
          </n-button>
        </div>
      </n-spin>
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
    title: 'ID',
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
    await connectionStore.setupMittListeners()
    refreshConnections()
  }
})

// 组件卸载时清理连接监听器
onUnmounted(() => {
  connectionStore.cleanupListeners()
})
</script>

<style scoped>
.connections-view {
  min-height: 100vh;
  background: var(--n-color-embedded);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

/* 英雄式头部 */
.hero-header {
  background: var(--n-card-color);
  border-radius: 20px;
  padding: 24px 32px;
  box-shadow: var(--n-box-shadow-2);
  border: 1px solid var(--n-border-color);
  position: relative;
  overflow: hidden;
}

.hero-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(90deg, #00b42a 0%, #4080ff 50%, #2266dd 100%);
  border-radius: 20px 20px 0 0;
}

.hero-content {
  display: flex;
  align-items: center;
  gap: 24px;
}

.hero-icon {
  width: 72px;
  height: 72px;
  border-radius: 18px;
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 12px 32px rgba(0, 180, 42, 0.3);
}

.hero-text {
  flex: 1;
}

.hero-title {
  font-size: 2rem;
  font-weight: 800;
  margin: 0 0 8px 0;
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  line-height: 1.2;
}

.hero-subtitle {
  font-size: 1.1rem;
  color: var(--n-text-color-3);
  margin: 0;
  font-weight: 500;
}

.hero-action {
  flex-shrink: 0;
}

.hero-btn {
  height: 48px;
  padding: 0 24px;
  font-size: 1rem;
  font-weight: 600;
  border-radius: 24px;
  box-shadow:
    0 8px 32px rgba(0, 180, 42, 0.25),
    0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.hero-btn:hover {
  transform: translateY(-3px);
  box-shadow:
    0 16px 48px rgba(0, 180, 42, 0.4),
    0 4px 12px rgba(0, 0, 0, 0.15);
}

/* 统计卡片网格 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}

.stat-card {
  background: var(--n-card-color);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.stat-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  border-radius: 14px 14px 0 0;
}

.stat-card.active-connections::before {
  background: linear-gradient(90deg, #00b42a 0%, #009a1a 100%);
}

.stat-card.upload-stats::before {
  background: linear-gradient(90deg, #f53f3f 0%, #cb2a2a 100%);
}

.stat-card.download-stats::before {
  background: linear-gradient(90deg, #4080ff 0%, #2266dd 100%);
}

.stat-card.memory-stats::before {
  background: linear-gradient(90deg, #f53f3f 0%, #cb2a2a 100%);
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow:
    0 16px 48px rgba(0, 0, 0, 0.12),
    0 4px 8px rgba(0, 0, 0, 0.08);
}

.stat-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  font-weight: 600;
}

.active-connections .stat-icon {
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
  color: white;
  box-shadow: 0 8px 24px rgba(0, 180, 42, 0.3);
}

.upload-stats .stat-icon {
  background: linear-gradient(135deg, #f53f3f 0%, #cb2a2a 100%);
  color: white;
  box-shadow: 0 8px 24px rgba(245, 63, 63, 0.3);
}

.download-stats .stat-icon {
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 100%);
  color: white;
  box-shadow: 0 8px 24px rgba(64, 128, 255, 0.3);
}

.memory-stats .stat-icon {
  background: linear-gradient(135deg, #f53f3f 0%, #cb2a2a 100%);
  color: white;
  box-shadow: 0 8px 24px rgba(245, 63, 63, 0.3);
}

.stat-content {
  flex: 1;
}

.stat-label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--n-text-color-3);
  margin-bottom: 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--n-text-color-1);
  line-height: 1.2;
}

/* 筛选卡片 */
.filter-card {
  background: var(--n-card-color);
  border-radius: 16px;
  padding: 24px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.filter-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.filter-title {
  font-size: 1.25rem;
  font-weight: 700;
  margin: 0;
}

.filter-stats {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-controls {
  display: flex;
  gap: 12px;
  align-items: stretch;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1;
  min-width: 300px;
}

.filter-selects {
  display: flex;
  gap: 12px;
  align-items: stretch;
  flex-wrap: wrap;
}

.filter-select {
  min-width: 180px;
}

.search-input :deep(.n-input) {
  border-radius: 12px;
  border: 2px solid var(--n-border-color);
  transition: all 0.3s ease;
}

.search-input :deep(.n-input:hover) {
  border-color: rgba(64, 128, 255, 0.3);
}

.search-input :deep(.n-input.n-input--focus) {
  border-color: #4080ff;
  box-shadow: 0 0 0 3px rgba(64, 128, 255, 0.1);
}

.filter-select :deep(.n-base-selection) {
  border-radius: 12px;
  border: 2px solid var(--n-border-color);
  transition: all 0.3s ease;
}

.filter-select :deep(.n-base-selection:hover) {
  border-color: rgba(64, 128, 255, 0.3);
}

.filter-select :deep(.n-base-selection.n-base-selection--focus) {
  border-color: #4080ff;
  box-shadow: 0 0 0 3px rgba(64, 128, 255, 0.1);
}

/* 表格卡片 */
.table-card {
  background: var(--n-card-color);
  border-radius: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  overflow: hidden;
}

.table-container {
  min-height: 400px;
  padding: 16px;
}

.connections-table :deep(.n-data-table) {
  border-radius: 0;
}

.connections-table :deep(.n-data-table-th) {
  background: var(--n-color-embedded);
  font-weight: 600;
  color: var(--n-text-color-1);
  border-bottom: 2px solid var(--n-border-color);
  padding: 12px 8px;
}

.connections-table :deep(.n-data-table-td) {
  padding: 8px;
  border-bottom: 1px solid var(--n-border-color);
}

.connections-table :deep(.n-data-table-tr:hover .n-data-table-td) {
  background: linear-gradient(135deg, rgba(64, 128, 255, 0.02) 0%, rgba(144, 147, 153, 0.02) 100%);
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 350px;
  padding: 48px 24px;
  text-align: center;
}

.empty-icon {
  color: var(--n-text-color-disabled);
  margin-bottom: 24px;
}

.empty-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--n-text-color-1);
  margin: 0 0 12px 0;
}

.empty-description {
  font-size: 1rem;
  color: var(--n-text-color-3);
  margin: 0 0 24px 0;
  line-height: 1.6;
  max-width: 400px;
}

.empty-action {
  margin-top: 24px;
}

/* 深色模式样式会通过CSS变量自动应用 */

/* 文本颜色会通过CSS变量自动适配暗色模式 */

/* 响应式设计 */
@media (max-width: 1024px) {
  .connections-view {
    padding: 16px;
    gap: 16px;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .connections-view {
    padding: 12px;
    gap: 16px;
  }

  .hero-header {
    padding: 20px;
    border-radius: 16px;
  }

  .hero-content {
    flex-direction: column;
    text-align: center;
    gap: 16px;
  }

  .hero-icon {
    width: 64px;
    height: 64px;
  }

  .hero-title {
    font-size: 1.75rem;
  }

  .hero-subtitle {
    font-size: 1rem;
  }

  .stats-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .stat-card {
    padding: 16px;
  }

  .filter-card {
    padding: 20px;
  }

  .filter-controls {
    flex-direction: column;
    gap: 12px;
  }

  .filter-selects {
    flex-direction: column;
    gap: 8px;
  }

  .search-input {
    min-width: unset;
  }

  .filter-select {
    min-width: unset;
  }

  .table-card {
    border-radius: 14px;
  }

  .connections-table :deep(.n-data-table) {
    font-size: 0.875rem;
  }

  .connections-table :deep(.n-data-table-th),
  .connections-table :deep(.n-data-table-td) {
    padding: 8px 6px;
  }
}

@media (max-width: 480px) {
  .connections-view {
    padding: 8px;
  }

  .hero-header {
    padding: 16px;
  }

  .hero-title {
    font-size: 1.5rem;
  }

  .hero-btn {
    height: 44px;
    padding: 0 20px;
    font-size: 0.875rem;
  }

  .stat-card {
    padding: 14px;
  }

  .filter-card {
    padding: 16px;
  }

  .table-container {
    padding: 12px;
  }
}
</style>
