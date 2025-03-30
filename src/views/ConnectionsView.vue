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
          <n-space justify="space-between" align="center">
            <n-statistic :label="t('connections.activeConnections')">
              {{ connections.length }}
            </n-statistic>
            <n-space>
              <n-statistic :label="t('connections.uploadTotal')">
                {{ formatBytes(connectionsTotal.upload) }}
              </n-statistic>
              <n-statistic :label="t('connections.downloadTotal')">
                {{ formatBytes(connectionsTotal.download) }}
              </n-statistic>
            </n-space>
          </n-space>
        </div>

        <div v-if="connections.length > 0" class="connections-list">
          <n-data-table
            :columns="columns"
            :data="connections"
            :pagination="pagination"
            :bordered="false"
            :max-height="600"
            striped
          />
        </div>
        <n-empty v-else :description="t('connections.noConnections')" />
      </n-spin>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed } from 'vue'
import { useMessage, NTag, DataTableColumns, NSpace, NTooltip, NText } from 'naive-ui'
import { RefreshOutline } from '@vicons/ionicons5'
import { useInfoStore } from '@/stores/infoStore'
import { useI18n } from 'vue-i18n'

const message = useMessage()
const loading = ref(false)
const infoStore = useInfoStore()
const { t } = useI18n()

// 使用计算属性来获取连接信息
const connections = computed(() => infoStore.connections)
const connectionsTotal = computed(() => infoStore.connectionsTotal)

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
              { default: () => network.toUpperCase() },
            ),
            h(
              NTag,
              {
                type: 'default',
                size: 'small',
                bordered: false,
              },
              { default: () => type },
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
      return h(
        NTooltip,
        {},
        {
          trigger: () => `${sourceIP}:${sourcePort}`,
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
      return h(
        NTooltip,
        {},
        {
          trigger: () => host || `${destinationIP}:${destinationPort}`,
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
    title: '进程',
    key: 'process',
    ellipsis: {
      tooltip: true,
    },
    render(row: Connection) {
      return row.metadata.processPath || '未知'
    },
  },
  {
    title: '流量',
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
  pageSize: 10,
}

// 刷新连接列表
const refreshConnections = async () => {
  loading.value = true
  try {
    // 这里实际上不需要做什么，因为infoStore中的connections已经通过WebSocket自动更新
    // 但我们仍然提供刷新按钮以便于用户手动刷新界面
    message.success('连接列表已刷新')
  } catch (error) {
    console.error('刷新连接列表失败:', error)
    message.error(`刷新连接列表失败: ${error}`)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  // 当组件挂载时，确保infoStore中的连接数据已经初始化
  if (!connections.value.length && infoStore.uptime > 0) {
    refreshConnections()
  }
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
  padding: 12px;
  background-color: var(--n-color-container);
  border-radius: 8px;
}

.connections-list {
  margin-top: 12px;
}
</style>
