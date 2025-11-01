import { defineStore } from 'pinia'
import { ref } from 'vue'
import { eventService } from '@/services/event-service'

// 定义连接状态接口
interface ConnectionState {
  connected: boolean
  connecting: boolean
  error: Error | null
}

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

interface ConnectionsData {
  connections: Connection[]
  downloadTotal: number
  uploadTotal: number
  memory: number
}

export const useConnectionStore = defineStore(
  'connection',
  () => {
    // 连接数据配置
    const MAX_CONNECTIONS = 500 // 最大保存连接数
    const CONNECTION_CLEANUP_THRESHOLD = 400 // 清理阈值
    const CONNECTION_RETAIN_COUNT = 200 // 清理后保留的连接数

    // 连接状态
    const connectionsState = ref<ConnectionState>({
      connected: false,
      connecting: false,
      error: null,
    })

    const memoryState = ref<ConnectionState>({
      connected: false,
      connecting: false,
      error: null,
    })

    // 连接信息
    const connections = ref<Connection[]>([])
    const connectionsTotal = ref({
      upload: 0,
      download: 0,
    })

    // 内存使用信息
    const memory = ref({
      inuse: 0,
      oslimit: 0,
      lastUpdated: Date.now(), // 添加最后更新时间戳
    })

    // 健康检查定时器
    let connectionsHealthCheck: number | null = null
    let memoryHealthCheck: number | null = null

    // 内存清理定时器
    let memoryCleanupTimer: number | null = null

    // 事件监听器状态
    let eventListenersSetup = false

    // 健康检查函数 - 连接数据
    const startConnectionsHealthCheck = () => {
      // 清除已有的定时器
      if (connectionsHealthCheck !== null) {
        clearInterval(connectionsHealthCheck)
      }

      // 设置新的定时器，每5秒检查一次
      connectionsHealthCheck = window.setInterval(() => {
        const lastConnection =
          connections.value.length > 0 ? connections.value[connections.value.length - 1] : null

        // 如果超过15秒没有新数据且状态为已连接，尝试重新连接
        if (
          connectionsState.value.connected &&
          (!lastConnection || Date.now() - new Date(lastConnection.start).getTime() > 15000)
        ) {
          console.log('🔄 连接数据长时间未更新，可能需要重新连接')
        }
      }, 5000)
    }

    // 健康检查函数 - 内存数据
    const startMemoryHealthCheck = () => {
      // 清除已有的定时器
      if (memoryHealthCheck !== null) {
        clearInterval(memoryHealthCheck)
      }

      // 设置新的定时器，每5秒检查一次
      memoryHealthCheck = window.setInterval(() => {
        // 如果超过10秒没有更新数据且状态为已连接，尝试重新连接
        if (memoryState.value.connected && Date.now() - memory.value.lastUpdated > 10000) {
          console.log('🔄 内存数据长时间未更新，可能需要重新连接')
        }
      }, 5000)
    }

    // 设置Tauri事件监听器
    const setupEventListeners = async () => {
      if (eventListenersSetup) return

      try {
        // 监听连接数据事件
        await eventService.onConnectionsData((data) => {
          // 类型检查
          if (data && typeof data === 'object' && 'connections' in data) {
            updateConnections(data as unknown as ConnectionsData)
            connectionsState.value.connected = true
            connectionsState.value.error = null
          }
        })

        // 监听内存数据事件
        await eventService.onMemoryData((data) => {
          // 类型检查
          if (data && typeof data === 'object' && 'inuse' in data && 'oslimit' in data) {
            updateMemory(data as unknown as { inuse: number; oslimit: number })
            memoryState.value.connected = true
            memoryState.value.error = null
          }
        })

        // 当收到任何数据时，说明连接正常
        connectionsState.value.connected = true
        memoryState.value.connected = true

        eventListenersSetup = true
        console.log('✅ 连接Store事件监听器设置完成')
      } catch (error) {
        console.error('❌ 连接Store事件监听器设置失败:', error)
      }
    }

    // 清理事件监听器
    const cleanupEventListeners = () => {
      if (!eventListenersSetup) return

      try {
        eventService.removeEventListener('connections-data')
        eventService.removeEventListener('memory-data')
      } catch (error) {
        console.error('清理连接监听器时出错:', error)
      } finally {
        eventListenersSetup = false
      }

      // 清除健康检查定时器
      if (connectionsHealthCheck !== null) {
        clearInterval(connectionsHealthCheck)
        connectionsHealthCheck = null
      }

      if (memoryHealthCheck !== null) {
        clearInterval(memoryHealthCheck)
        memoryHealthCheck = null
      }
    }

    // 重置连接数据
    const resetData = () => {
      connections.value = []
      connectionsTotal.value = {
        upload: 0,
        download: 0,
      }
      memory.value = {
        inuse: 0,
        oslimit: 0,
        lastUpdated: Date.now(),
      }
      connectionsState.value = {
        connected: false,
        connecting: false,
        error: null,
      }
      memoryState.value = {
        connected: false,
        connecting: false,
        error: null,
      }
    }

    // 智能连接数据清理
    const smartConnectionCleanup = () => {
      if (connections.value.length <= CONNECTION_CLEANUP_THRESHOLD) {
        return // 未达到清理阈值
      }

      // 按时间排序，保留最新的连接
      const sortedConnections = [...connections.value].sort(
        (a, b) => new Date(b.start).getTime() - new Date(a.start).getTime(),
      )

      connections.value = sortedConnections.slice(0, CONNECTION_RETAIN_COUNT)
      console.log(`🧹 清理连接数据，保留 ${connections.value.length} 条最新连接`)
    }

    // 启动内存监控
    const startMemoryMonitoring = () => {
      if (memoryCleanupTimer) {
        clearInterval(memoryCleanupTimer)
      }

      memoryCleanupTimer = window.setInterval(() => {
        // 检查连接数量并进行清理
        if (connections.value.length >= CONNECTION_CLEANUP_THRESHOLD) {
          smartConnectionCleanup()
        }

        // 检查内存数据时效性
        const now = Date.now()
        if (now - memory.value.lastUpdated > 60000) {
          // 1分钟无更新
          // 可能需要重新连接内存监控
          if (memoryState.value.connected) {
            console.log('🔄 内存数据长时间未更新，可能需要重新连接')
          }
        }
      }, 30 * 1000) // 30秒检查一次
    }

    // 停止内存监控
    const stopMemoryMonitoring = () => {
      if (memoryCleanupTimer) {
        clearInterval(memoryCleanupTimer)
        memoryCleanupTimer = null
      }
    }

    // 更新连接数据（优化版本）
    const updateConnections = (data: ConnectionsData) => {
      try {
        if (data?.connections && Array.isArray(data.connections)) {
          // 限制连接数量以防止内存溢出
          const newConnections = data.connections.slice(0, MAX_CONNECTIONS)
          connections.value = newConnections

          connectionsTotal.value = {
            upload: data.uploadTotal || 0,
            download: data.downloadTotal || 0,
          }
        }
      } catch (error) {
        console.error('更新连接数据失败:', error)
      }
    }

    // 更新内存数据（优化版本）
    const updateMemory = (data: { inuse: number; oslimit: number }) => {
      try {
        if (data && typeof data.inuse === 'number' && typeof data.oslimit === 'number') {
          memory.value = {
            inuse: data.inuse,
            oslimit: data.oslimit,
            lastUpdated: Date.now(),
          }
        }
      } catch (error) {
        console.error('更新内存数据失败:', error)
      }
    }

    return {
      // 状态
      connectionsState,
      memoryState,

      // 数据
      connections,
      connectionsTotal,
      memory,

      // 方法
      setupEventListeners,
      cleanupEventListeners,
      resetData,
      updateConnections,
      updateMemory,
      smartConnectionCleanup,
      startMemoryMonitoring,
      stopMemoryMonitoring,
      startConnectionsHealthCheck,
      startMemoryHealthCheck,
    }
  },
)