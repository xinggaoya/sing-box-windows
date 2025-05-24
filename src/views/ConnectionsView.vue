<template>
  <div class="connections-view">
    <!-- 主要内容区 -->
    <div class="page-container">
      <!-- 页面标题栏 -->
      <div class="page-header">
        <div class="header-content">
          <div class="title-section">
            <h1 class="page-title">{{ t('connections.title') }}</h1>
            <div class="title-divider"></div>
          </div>
          <n-button
            type="primary"
            @click="refreshConnections"
            :loading="loading"
            class="refresh-btn"
            size="large"
            round
          >
            <template #icon>
              <n-icon><refresh-outline /></n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
        </div>
      </div>

      <!-- 统计卡片区 -->
      <div class="stats-section">
        <div class="stat-card active-connections">
          <div class="stat-icon">
            <n-icon size="28"><link-outline /></n-icon>
          </div>
          <div class="stat-content">
            <div class="stat-label">{{ t('connections.activeConnections') }}</div>
            <div class="stat-value">
              <n-number-animation ref="activeCountRef" :from="0" :to="connections.length" />
            </div>
          </div>
        </div>

        <div class="stat-card upload-stats">
          <div class="stat-icon">
            <n-icon size="28"><arrow-up-outline /></n-icon>
          </div>
          <div class="stat-content">
            <div class="stat-label">{{ t('connections.uploadTotal') }}</div>
            <div class="stat-value">{{ formatBytes(connectionsTotal.upload) }}</div>
          </div>
        </div>

        <div class="stat-card download-stats">
          <div class="stat-icon">
            <n-icon size="28"><arrow-down-outline /></n-icon>
          </div>
          <div class="stat-content">
            <div class="stat-label">{{ t('connections.downloadTotal') }}</div>
            <div class="stat-value">{{ formatBytes(connectionsTotal.download) }}</div>
          </div>
        </div>
      </div>

      <!-- 搜索筛选区 -->
      <div class="filter-section">
        <div class="filter-card">
          <div class="filter-row">
            <n-input
              v-model:value="searchQuery"
              placeholder="搜索连接..."
              clearable
              class="search-input"
              size="large"
            >
              <template #prefix>
                <n-icon><search-outline /></n-icon>
              </template>
            </n-input>

            <n-select
              v-model:value="networkFilter"
              :options="networkOptions"
              placeholder="网络类型"
              clearable
              class="filter-select"
              size="large"
            />

            <n-select
              v-model:value="ruleFilter"
              :options="ruleOptions"
              placeholder="规则筛选"
              clearable
              class="filter-select"
              size="large"
            />
          </div>

          <!-- 筛选统计 -->
          <div class="filter-stats">
            <n-tag type="default" size="small" class="stat-tag">
              总计: {{ connections.length }}
            </n-tag>
            <n-tag
              v-if="searchQuery || networkFilter || ruleFilter"
              type="primary"
              size="small"
              class="stat-tag"
            >
              匹配: {{ filteredConnections.length }}
            </n-tag>
          </div>
        </div>
      </div>

      <!-- 连接列表区 -->
      <div class="connections-section">
        <n-spin :show="loading" class="table-container">
          <div v-if="filteredConnections.length > 0" class="table-wrapper">
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
            <div class="empty-content">
              <n-icon size="48" class="empty-icon">
                <link-outline />
              </n-icon>
              <h3 class="empty-title">
                {{ searchQuery || networkFilter || ruleFilter ? '无匹配连接' : '暂无连接' }}
              </h3>
              <p class="empty-description">
                {{
                  searchQuery || networkFilter || ruleFilter
                    ? '尝试调整筛选条件'
                    : '当前没有活跃的网络连接'
                }}
              </p>
            </div>
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
} from '@vicons/ionicons5'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useI18n } from 'vue-i18n'

const message = useMessage()
const loading = ref(false)
const connectionStore = useConnectionStore()
const { t } = useI18n()
const activeCountRef = ref(null)

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
    await connectionStore.setupConnectionsListener()
    await connectionStore.setupMemoryListener()
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
  background: linear-gradient(
    135deg,
    rgba(64, 128, 255, 0.02) 0%,
    rgba(144, 147, 153, 0.02) 35%,
    rgba(0, 180, 42, 0.02) 100%
  );
  padding: 0;
}

.page-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 24px 20px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* 页面标题栏 */
.page-header {
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.9) 0%, rgba(248, 250, 252, 0.8) 100%);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  padding: 24px 32px;
  box-shadow:
    0 10px 40px rgba(0, 0, 0, 0.1),
    0 1px 3px rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 24px;
}

.title-section {
  display: flex;
  align-items: center;
  gap: 16px;
}

.page-title {
  font-size: 2rem;
  font-weight: 700;
  margin: 0;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 50%, #009a1a 100%);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.title-divider {
  width: 4px;
  height: 32px;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 50%, #009a1a 100%);
  border-radius: 2px;
}

.refresh-btn {
  font-weight: 600;
  padding: 0 24px;
  height: 48px;
  box-shadow:
    0 8px 32px rgba(64, 128, 255, 0.3),
    0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.refresh-btn:hover {
  transform: translateY(-2px);
  box-shadow:
    0 12px 40px rgba(64, 128, 255, 0.4),
    0 4px 8px rgba(0, 0, 0, 0.15);
}

/* 统计卡片区 */
.stats-section {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
}

.stat-card {
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.95) 0%, rgba(248, 250, 252, 0.9) 100%);
  backdrop-filter: blur(20px);
  border-radius: 18px;
  padding: 24px;
  display: flex;
  align-items: center;
  gap: 20px;
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.08),
    0 1px 3px rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.3);
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
  border-radius: 18px 18px 0 0;
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

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow:
    0 16px 48px rgba(0, 0, 0, 0.12),
    0 4px 8px rgba(0, 0, 0, 0.08);
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
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

.stat-content {
  flex: 1;
}

.stat-label {
  font-size: 0.875rem;
  font-weight: 500;
  color: rgba(107, 114, 128, 0.8);
  margin-bottom: 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  font-size: 1.75rem;
  font-weight: 700;
  color: #1f2937;
  line-height: 1.2;
}

/* 筛选区 */
.filter-section {
  margin: 0;
}

.filter-card {
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.95) 0%, rgba(248, 250, 252, 0.9) 100%);
  backdrop-filter: blur(20px);
  border-radius: 18px;
  padding: 24px;
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.08),
    0 1px 3px rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.3);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.filter-row {
  display: flex;
  gap: 16px;
  align-items: stretch;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1;
  min-width: 300px;
}

.filter-select {
  min-width: 180px;
}

.search-input :deep(.n-input) {
  border-radius: 12px;
  border: 2px solid rgba(229, 231, 235, 0.5);
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
  border: 2px solid rgba(229, 231, 235, 0.5);
  transition: all 0.3s ease;
}

.filter-select :deep(.n-base-selection:hover) {
  border-color: rgba(64, 128, 255, 0.3);
}

.filter-select :deep(.n-base-selection.n-base-selection--focus) {
  border-color: #4080ff;
  box-shadow: 0 0 0 3px rgba(64, 128, 255, 0.1);
}

.filter-stats {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.stat-tag {
  font-weight: 600;
  padding: 6px 12px;
  border-radius: 8px;
}

/* 连接列表区 */
.connections-section {
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.95) 0%, rgba(248, 250, 252, 0.9) 100%);
  backdrop-filter: blur(20px);
  border-radius: 18px;
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.08),
    0 1px 3px rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.3);
  overflow: hidden;
}

.table-container {
  min-height: 300px;
}

.table-wrapper {
  padding: 0;
}

.connections-table :deep(.n-data-table) {
  border-radius: 0;
}

.connections-table :deep(.n-data-table-th) {
  background: linear-gradient(135deg, rgba(249, 250, 251, 0.9) 0%, rgba(243, 244, 246, 0.8) 100%);
  font-weight: 600;
  color: #374151;
  border-bottom: 2px solid rgba(229, 231, 235, 0.5);
  padding: 16px 12px;
}

.connections-table :deep(.n-data-table-td) {
  padding: 12px;
  border-bottom: 1px solid rgba(229, 231, 235, 0.3);
}

.connections-table :deep(.n-data-table-tr:hover .n-data-table-td) {
  background: linear-gradient(135deg, rgba(64, 128, 255, 0.02) 0%, rgba(144, 147, 153, 0.02) 100%);
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  padding: 48px 24px;
}

.empty-content {
  text-align: center;
  max-width: 400px;
}

.empty-icon {
  color: rgba(156, 163, 175, 0.6);
  margin-bottom: 24px;
}

.empty-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: #374151;
  margin: 0 0 12px 0;
}

.empty-description {
  font-size: 0.875rem;
  color: rgba(107, 114, 128, 0.8);
  margin: 0;
  line-height: 1.5;
}

/* 深色模式支持 */
@media (prefers-color-scheme: dark) {
  .connections-view {
    background: linear-gradient(
      135deg,
      rgba(17, 24, 39, 0.95) 0%,
      rgba(31, 41, 55, 0.9) 35%,
      rgba(55, 65, 81, 0.85) 100%
    );
  }

  .page-header,
  .stat-card,
  .filter-card,
  .connections-section {
    background: linear-gradient(135deg, rgba(31, 41, 55, 0.95) 0%, rgba(17, 24, 39, 0.9) 100%);
    border-color: rgba(75, 85, 99, 0.3);
  }

  .page-title {
    color: white;
    -webkit-text-fill-color: unset;
    background: unset;
    background-clip: unset;
    -webkit-background-clip: unset;
  }

  .stat-label {
    color: rgba(156, 163, 175, 0.8);
  }

  .stat-value {
    color: #f9fafb;
  }

  .empty-title {
    color: #f9fafb;
  }

  .connections-table :deep(.n-data-table-th) {
    background: linear-gradient(135deg, rgba(55, 65, 81, 0.9) 0%, rgba(31, 41, 55, 0.8) 100%);
    color: #f9fafb;
    border-bottom-color: rgba(75, 85, 99, 0.5);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .page-container {
    padding: 16px 12px;
    gap: 20px;
  }

  .page-header {
    padding: 20px 24px;
    border-radius: 16px;
  }

  .header-content {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;
  }

  .page-title {
    font-size: 1.5rem;
    text-align: center;
  }

  .stats-section {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  .stat-card {
    padding: 20px;
    border-radius: 16px;
  }

  .filter-row {
    flex-direction: column;
    gap: 12px;
  }

  .search-input {
    min-width: unset;
  }

  .filter-select {
    min-width: unset;
  }

  .connections-section {
    border-radius: 16px;
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
  .page-container {
    padding: 12px 8px;
  }

  .stat-card {
    padding: 16px;
  }

  .stat-icon {
    width: 48px;
    height: 48px;
  }

  .stat-value {
    font-size: 1.5rem;
  }
}
</style>
