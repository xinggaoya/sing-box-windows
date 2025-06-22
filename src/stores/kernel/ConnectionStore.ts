import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import mitt from '@/utils/mitt'
import { temporaryStoreManager } from '@/utils/memory-leak-fix'

import { WebSocketService, ConnectionState } from '@/services/websocket-service'

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
    // WebSocket 服务实例
    const wsService = WebSocketService.getInstance()

    // 连接数据配置（更严格的限制）
    const MAX_CONNECTIONS = 200 // 减少最大保存连接数，从500减少到200
    const CONNECTION_CLEANUP_THRESHOLD = 150 // 减少清理阈值，从400减少到150
    const CONNECTION_RETAIN_COUNT = 100 // 减少保留数量，从200减少到100

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

    // 存储事件监听器清理函数
    let unlistenConnectionsStateFn: (() => void) | null = null
    let unlistenConnectionsDataFn: (() => void) | null = null
    let unlistenMemoryStateFn: (() => void) | null = null
    let unlistenMemoryDataFn: (() => void) | null = null

    // Mitt事件监听器状态
    let mittListenersRegistered = false

    // 健康检查函数 - 连接数据（优化版本）
    const startConnectionsHealthCheck = () => {
      // 清除已有的定时器
      if (connectionsHealthCheck !== null) {
        clearInterval(connectionsHealthCheck)
      }

      // 降低检查频率：每30秒检查一次，而不是5秒
      connectionsHealthCheck = window.setInterval(() => {
        // 只有在明确连接但长时间无数据时才重连
        const shouldReconnect =
          connectionsState.value.connected &&
          connections.value.length === 0 && // 完全没有连接数据
          Date.now() - memory.value.lastUpdated > 60000 // 超过1分钟没有任何数据更新

        if (shouldReconnect) {
          console.log('🔄 连接健康检查：长时间无数据，尝试重连')
          reconnectConnectionsWebSocket()
        }
      }, 30000) // 30秒检查一次
    }

    // 健康检查函数 - 内存数据（优化版本）
    const startMemoryHealthCheck = () => {
      // 清除已有的定时器
      if (memoryHealthCheck !== null) {
        clearInterval(memoryHealthCheck)
      }

      // 降低检查频率：每30秒检查一次
      memoryHealthCheck = window.setInterval(() => {
        // 只有在长时间没有内存数据更新时才重连
        const shouldReconnect =
          memoryState.value.connected && Date.now() - memory.value.lastUpdated > 120000 // 超过2分钟没有内存数据更新

        if (shouldReconnect) {
          console.log('🔄 内存健康检查：长时间无数据，尝试重连')
          reconnectMemoryWebSocket()
        }
      }, 30000) // 30秒检查一次
    }

    // 重新连接连接WebSocket（优化版本）
    const reconnectConnectionsWebSocket = async () => {
      try {
        console.log('🔌 重新连接连接WebSocket...')

        // 断开现有连接
        await wsService.disconnect('connections')

        // 增加延迟，避免频繁重连：3秒而不是1秒
        setTimeout(async () => {
          try {
            const success = await wsService.connect('connections')
            if (success) {
              console.log('✅ 连接WebSocket重连成功')
            } else {
              console.log('❌ 连接WebSocket重连失败')
            }
          } catch (error) {
            console.error('连接WebSocket重连异常:', error)
          }
        }, 3000)
      } catch (error) {
        console.error('重新连接连接WebSocket失败:', error)
      }
    }

    // 重新连接内存WebSocket（优化版本）
    const reconnectMemoryWebSocket = async () => {
      try {
        console.log('🧠 重新连接内存WebSocket...')

        // 断开现有连接
        await wsService.disconnect('memory')

        // 增加延迟，避免频繁重连：3秒而不是1秒
        setTimeout(async () => {
          try {
            const success = await wsService.connect('memory')
            if (success) {
              console.log('✅ 内存WebSocket重连成功')
            } else {
              console.log('❌ 内存WebSocket重连失败')
            }
          } catch (error) {
            console.error('内存WebSocket重连异常:', error)
          }
        }, 3000)
      } catch (error) {
        console.error('重新连接内存WebSocket失败:', error)
      }
    }

    // 设置Mitt事件监听器
    const setupMittListeners = () => {
      if (mittListenersRegistered) return

      // 监听连接数据事件
      mitt.on('connections-data', (data) => {
        // 类型检查
        if (data && typeof data === 'object' && 'connections' in data) {
          updateConnections(data as unknown as ConnectionsData)
          connectionsState.value.connected = true
          connectionsState.value.error = null
        }
      })

      // 监听内存数据事件
      mitt.on('memory-data', (data) => {
        // 类型检查
        if (data && typeof data === 'object' && 'inuse' in data && 'oslimit' in data) {
          updateMemory(data as unknown as { inuse: number; oslimit: number })
          memoryState.value.connected = true
          memoryState.value.error = null
        }
      })

      // 监听WebSocket连接状态
      mitt.on('ws-connected', () => {
        connectionsState.value.connected = true
        memoryState.value.connected = true
      })

      mitt.on('ws-disconnected', () => {
        connectionsState.value.connected = false
        memoryState.value.connected = false
      })

      mittListenersRegistered = true
    }

    // 清理Mitt监听器
    const cleanupMittListeners = () => {
      if (!mittListenersRegistered) return

      mitt.off('connections-data')
      mitt.off('memory-data')
      mitt.off('ws-connected')
      mitt.off('ws-disconnected')

      mittListenersRegistered = false
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

    // 清理所有监听器
    const cleanupListeners = () => {
      // 清理Mitt监听器
      cleanupMittListeners()

      // 清理Tauri监听器
      if (unlistenConnectionsStateFn) {
        unlistenConnectionsStateFn()
        unlistenConnectionsStateFn = null
      }

      if (unlistenConnectionsDataFn) {
        unlistenConnectionsDataFn()
        unlistenConnectionsDataFn = null
      }

      if (unlistenMemoryStateFn) {
        unlistenMemoryStateFn()
        unlistenMemoryStateFn = null
      }

      if (unlistenMemoryDataFn) {
        unlistenMemoryDataFn()
        unlistenMemoryDataFn = null
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

    // 启动内存监控（优化版本）
    const startMemoryMonitoring = () => {
      if (memoryCleanupTimer) {
        clearInterval(memoryCleanupTimer)
      }

      // 降低监控频率：从30秒改为60秒
      memoryCleanupTimer = window.setInterval(() => {
        // 检查连接数量并进行清理
        if (connections.value.length >= CONNECTION_CLEANUP_THRESHOLD) {
          console.log(`🧹 连接数量达到 ${connections.value.length}，开始清理`)
          smartConnectionCleanup()
        }

        // 检查内存数据时效性（放宽检查条件）
        const now = Date.now()
        if (now - memory.value.lastUpdated > 300000) {
          // 5分钟无更新，之前是1分钟
          // 可能需要重新连接内存监控
          if (memoryState.value.connected) {
            console.log('🔄 内存数据长时间未更新，尝试重新连接')
            reconnectMemoryWebSocket()
          }
        }
      }, 60 * 1000) // 60秒检查一次，之前是30秒
    }

    // 停止内存监控
    const stopMemoryMonitoring = () => {
      if (memoryCleanupTimer) {
        clearInterval(memoryCleanupTimer)
        memoryCleanupTimer = null
      }
    }

    // 更新连接数据（优化版本，减少频繁更新）
    const updateConnections = (data: ConnectionsData) => {
      try {
        if (data?.connections && Array.isArray(data.connections)) {
          // 进一步限制连接数量以防止内存溢出
          const newConnections = data.connections.slice(0, MAX_CONNECTIONS)

          // 只有在连接数据有明显变化时才更新
          const hasSignificantChange =
            Math.abs(connections.value.length - newConnections.length) > 5 || // 连接数变化超过5个
            connections.value.length === 0 // 或者当前没有连接数据

          if (hasSignificantChange) {
            connections.value = newConnections
            console.log(`📊 更新连接数据：${newConnections.length} 个连接`)
          }

          // 总计数据总是更新
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

    // Store初始化方法
    const initializeStore = () => {
      setupMittListeners()
      startMemoryMonitoring()
      startConnectionsHealthCheck()
      startMemoryHealthCheck()

      // 注册到临时Store管理器
      const storeInstance = {
        cleanupStore,
        smartConnectionCleanup,
      }
      temporaryStoreManager.registerStore('connection', storeInstance)
    }

    // Store清理方法
    const cleanupStore = () => {
      cleanupListeners()
      stopMemoryMonitoring()
      resetData()

      // 从临时Store管理器注销
      temporaryStoreManager.unregisterStore('connection')
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
      setupMittListeners,
      cleanupMittListeners,
      cleanupListeners,
      resetData,
      reconnectConnectionsWebSocket,
      reconnectMemoryWebSocket,
      updateConnections,
      updateMemory,
      smartConnectionCleanup,
      startMemoryMonitoring,
      stopMemoryMonitoring,
      initializeStore,
      cleanupStore,
    }
  },
  {
    // 连接数据不需要持久化存储 - 实时数据应在应用重启时重置
    persist: false,
  },
)
