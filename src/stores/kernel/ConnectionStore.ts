import { defineStore } from 'pinia'
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import mitt from '@/utils/mitt'
import { StoreCleaner } from '@/utils/memory-leak-fix'
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

    // 存储事件监听器清理函数
    let unlistenConnectionsStateFn: (() => void) | null = null
    let unlistenConnectionsDataFn: (() => void) | null = null
    let unlistenMemoryStateFn: (() => void) | null = null
    let unlistenMemoryDataFn: (() => void) | null = null

    // Mitt事件监听器状态
    let mittListenersRegistered = false

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
          console.log('连接数据超过15秒未更新，尝试重新连接...')
          reconnectConnectionsWebSocket()
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
          console.log('内存数据超过10秒未更新，尝试重新连接...')
          reconnectMemoryWebSocket()
        }
      }, 5000)
    }

    // 重新连接连接WebSocket
    const reconnectConnectionsWebSocket = async () => {
      try {
        // 断开现有连接
        await wsService.disconnect('connections')
        // 短暂延迟后重新连接
        setTimeout(async () => {
          await wsService.connect('connections')
        }, 1000)
      } catch (error) {
        console.error('重新连接连接WebSocket失败:', error)
      }
    }

    // 重新连接内存WebSocket
    const reconnectMemoryWebSocket = async () => {
      try {
        // 断开现有连接
        await wsService.disconnect('memory')
        // 短暂延迟后重新连接
        setTimeout(async () => {
          await wsService.connect('memory')
        }, 1000)
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
      console.log('🔗 ConnectionStore Mitt监听器已注册')
    }

    // 清理Mitt监听器
    const cleanupMittListeners = () => {
      if (!mittListenersRegistered) return

      mitt.off('connections-data')
      mitt.off('memory-data')
      mitt.off('ws-connected')
      mitt.off('ws-disconnected')

      mittListenersRegistered = false
      console.log('🧹 ConnectionStore Mitt监听器已清理')
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

      console.log('🧹 ConnectionStore 监听器已清理')
    }

    // 更新连接数据
    const updateConnections = (data: ConnectionsData) => {
      if (data && 'connections' in data) {
        try {
          // 更新连接列表
          connections.value = data.connections || []

          // 更新统计数据，确保是数值
          connectionsTotal.value = {
            upload: Number(data.uploadTotal) || 0,
            download: Number(data.downloadTotal) || 0,
          }

          // 如果数据接收正常，但当前状态不是连接状态，更新状态
          if (!connectionsState.value.connected) {
            connectionsState.value.connected = true
            connectionsState.value.connecting = false
            connectionsState.value.error = null
          }
        } catch (error) {
          console.error('处理连接数据时出错:', error, data)
        }
      }
    }

    // 更新内存数据
    const updateMemory = (data: { inuse: number; oslimit: number }) => {
      if ('inuse' in data && 'oslimit' in data) {
        try {
          // 确保数据是数值类型
          const inuse = Number(data.inuse) || 0
          const oslimit = Number(data.oslimit) || 0

          memory.value = {
            inuse,
            oslimit,
            lastUpdated: Date.now(), // 更新时间戳
          }

          // 如果数据接收正常，但当前状态不是连接状态，更新状态
          if (!memoryState.value.connected) {
            memoryState.value.connected = true
            memoryState.value.connecting = false
            memoryState.value.error = null
          }
        } catch (error) {
          console.error('处理内存数据时出错:', error, data)
        }
      }
    }

    // 监听内存清理请求
    mitt.on('memory-cleanup-requested', () => {
      console.log('🧹 响应内存清理请求 - Connection Store')

      // 清理旧连接数据
      if (connections.value.length > 100) {
        connections.value = connections.value.slice(0, 50)
        console.log('🧹 清理了过多的连接数据')
      }

      // 重置计数器
      connectionsTotal.value = { upload: 0, download: 0 }
    })

    // 注册清理函数
    StoreCleaner.registerCleanup(() => {
      cleanupListeners()
      resetData()
    })

    // 组件挂载时初始化
    onMounted(() => {
      setupMittListeners()
    })

    // 组件卸载时清理
    onUnmounted(() => {
      cleanupListeners()
    })

    return {
      connections,
      connectionsTotal,
      memory,
      connectionsState,
      memoryState,
      updateConnections,
      updateMemory,
      setupMittListeners,
      cleanupMittListeners,
      cleanupListeners,
      resetData,
      reconnectConnectionsWebSocket,
      reconnectMemoryWebSocket,
    }
  },
  {
    persist: false, // 不持久化，避免内存泄漏
  },
)
