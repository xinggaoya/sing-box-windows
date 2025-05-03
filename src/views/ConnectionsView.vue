<template>
  <div class="connections-container">
    <n-card class="connections-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <h2>{{ t('connections.title') }}</h2>
          <n-space>
            <n-button type="primary" @click="refreshConnections" :loading="loading">
              <template #icon>
                <n-icon><refresh-outline /></n-icon>
              </template>
              {{ t('common.refresh') }}
            </n-button>
          </n-space>
        </div>
      </template>

      <n-spin :show="loading">
        <div class="stats-bar">
          <n-grid :cols="3" :x-gap="12">
            <n-gi>
              <n-statistic :label="t('connections.activeConnections')" class="stat-item">
                <template #prefix>
                  <n-icon color="#18a058"><link-outline /></n-icon>
                </template>
                <n-number-animation ref="activeCountRef" :from="0" :to="connections.length" />
              </n-statistic>
            </n-gi>
            <n-gi>
              <n-statistic :label="t('connections.uploadTotal')" class="stat-item">
                <template #prefix>
                  <n-icon color="#d03050"><arrow-up-outline /></n-icon>
                </template>
                {{ formatBytes(connectionsTotal.upload) }}
              </n-statistic>
            </n-gi>
            <n-gi>
              <n-statistic :label="t('connections.downloadTotal')" class="stat-item">
                <template #prefix>
                  <n-icon color="#2080f0"><arrow-down-outline /></n-icon>
                </template>
                {{ formatBytes(connectionsTotal.download) }}
              </n-statistic>
            </n-gi>
          </n-grid>
        </div>

        <div class="search-filter-bar">
          <n-input
            v-model:value="searchQuery"
            placeholder="搜索连接..."
            clearable
            :style="{ width: '300px' }"
          >
            <template #prefix>
              <n-icon><search-outline /></n-icon>
            </template>
          </n-input>
          <n-select
            v-model:value="networkFilter"
            :options="networkOptions"
            placeholder="按网络类型筛选"
            clearable
            :style="{ width: '180px' }"
          />
          <n-select
            v-model:value="ruleFilter"
            :options="ruleOptions"
            placeholder="按规则筛选"
            clearable
            :style="{ width: '180px' }"
          />
        </div>

        <div v-if="filteredConnections.length > 0" class="connections-list">
          <n-data-table
            :columns="columns"
            :data="filteredConnections"
            :pagination="pagination"
            :bordered="false"
            :max-height="600"
            striped
          />
        </div>
        <n-empty v-else :description="searchQuery || networkFilter || ruleFilter ? t('connections.noMatchingConnections') : t('connections.noConnections')" />
        
        <div class="filter-stats">
          <n-tag type="info" size="small">{{ t('connections.totalConnections') }}: {{ connections.length }}</n-tag>
          <n-tag v-if="searchQuery || networkFilter || ruleFilter" type="success" size="small">
            {{ t('connections.matchingConnections') }}: {{ filteredConnections.length }}
          </n-tag>
        </div>
      </n-spin>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, h, computed, watch } from 'vue'
import { useMessage, NTag, DataTableColumns, NSpace, NTooltip, NText, SelectOption } from 'naive-ui'
import { RefreshOutline, SearchOutline, LinkOutline, ArrowUpOutline, ArrowDownOutline } from '@vicons/ionicons5'
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
  return connections.value.filter(conn => {
    const matchesSearch = !searchQuery.value || 
      conn.id.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      (conn.metadata.host?.toLowerCase() || '').includes(searchQuery.value.toLowerCase()) ||
      conn.metadata.destinationIP.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      conn.metadata.sourceIP.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      (conn.rule && String(conn.rule).toLowerCase().includes(searchQuery.value.toLowerCase())) ||
      (conn.metadata.processPath?.toLowerCase() || '').includes(searchQuery.value.toLowerCase())
    
    const matchesNetwork = !networkFilter.value || 
      (conn.metadata.network && networkFilter.value && String(conn.metadata.network).toLowerCase() === String(networkFilter.value).toLowerCase())
    
    const matchesRule = !ruleFilter.value || 
      (conn.rule && ruleFilter.value && String(conn.rule).toLowerCase() === String(ruleFilter.value).toLowerCase())
    
    return matchesSearch && matchesNetwork && matchesRule
  })
})

// 网络类型选项
const networkOptions = computed(() => {
  const networks = new Set<string>()
  connections.value.forEach(conn => {
    if (conn.metadata.network) {
      networks.add(String(conn.metadata.network).toUpperCase())
    }
  })
  return Array.from(networks).map(network => ({ label: network, value: network.toLowerCase() }))
})

// 规则选项
const ruleOptions = computed(() => {
  const rules = new Set<string>()
  connections.value.forEach(conn => {
    if (conn.rule) {
      rules.add(conn.rule)
    }
  })
  return Array.from(rules).map(rule => ({ label: rule, value: rule }))
})

// 监听连接变化以更新动画
watch(() => connections.value.length, (newVal, oldVal) => {
  if (activeCountRef.value && newVal !== oldVal) {
    // @ts-ignore - 忽略类型错误，因为 NNumberAnimation 组件确实有 play 方法
    activeCountRef.value.play()
  }
})

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
          h('span', { style: { backgroundColor: 'rgba(var(--primary-color), 0.1)', fontWeight: 'bold' } }, match),
          afterMatch
        ])
      }
      return row.id
    }
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
              { default: () => (typeof network === 'string' ? network.toUpperCase() : String(network).toUpperCase()) },
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
            trigger: () => h('div', {}, [
              beforeMatch,
              h('span', { style: { backgroundColor: 'rgba(var(--primary-color), 0.1)', fontWeight: 'bold' } }, match),
              afterMatch
            ]),
            default: () =>
              h('div', {}, [
                h('div', {}, `${t('connections.ip')}: ${sourceIP}`),
                h('div', {}, `${t('connections.port')}: ${sourcePort}`),
              ]),
          }
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
            trigger: () => h('div', {}, [
              beforeMatch,
              h('span', { style: { backgroundColor: 'rgba(var(--primary-color), 0.1)', fontWeight: 'bold' } }, match),
              afterMatch
            ]),
            default: () =>
              h('div', {}, [
                host ? h('div', {}, `${t('connections.host')}: ${host}`) : null,
                h('div', {}, `${t('connections.ip')}: ${destinationIP}`),
                h('div', {}, `${t('connections.port')}: ${destinationPort}`),
              ]),
          }
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
      if (searchQuery.value && row.rule && String(row.rule).toLowerCase().includes(searchQuery.value.toLowerCase())) {
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
                  default: () => h('div', {}, [
                    beforeMatch,
                    h('span', { style: { backgroundColor: 'rgba(var(--primary-color), 0.1)', fontWeight: 'bold' } }, match),
                    afterMatch
                  ])
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
      if (searchQuery.value && processPath.toLowerCase().includes(searchQuery.value.toLowerCase())) {
        const index = processPath.toLowerCase().indexOf(searchQuery.value.toLowerCase())
        const beforeMatch = processPath.substring(0, index)
        const match = processPath.substring(index, index + searchQuery.value.length)
        const afterMatch = processPath.substring(index + searchQuery.value.length)
        
        return h('div', {}, [
          beforeMatch,
          h('span', { style: { backgroundColor: 'rgba(var(--primary-color), 0.1)', fontWeight: 'bold' } }, match),
          afterMatch
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
.connections-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 12px 8px;
}

.connections-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
}

.connections-card :deep(.n-card__content) {
  padding: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 500;
}

.stats-bar {
  margin-bottom: 16px;
  padding: 16px;
  background-color: var(--card-color, rgba(var(--primary-color), 0.05));
  border-radius: 12px;
}

.stat-item {
  padding: 8px;
  text-align: center;
}

.search-filter-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.connections-list {
  margin-top: 12px;
}

.filter-stats {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  padding: 8px 0;
}
</style>
