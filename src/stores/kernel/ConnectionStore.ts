import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
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
      error: null
    })
    
    const memoryState = ref<ConnectionState>({
      connected: false,
      connecting: false,
      error: null
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
      lastUpdated: Date.now() // 添加最后更新时间戳
    })

    // 存储事件监听器清理函数
    let unlistenConnectionsStateFn: (() => void) | null = null
    let unlistenConnectionsDataFn: (() => void) | null = null
    let unlistenMemoryStateFn: (() => void) | null = null
    let unlistenMemoryDataFn: (() => void) | null = null
    
    // 健康检查定时器
    let connectionsHealthCheck: number | null = null
    let memoryHealthCheck: number | null = null

    // 健康检查函数 - 连接数据
    const startConnectionsHealthCheck = () => {
      // 清除已有的定时器
      if (connectionsHealthCheck !== null) {
        clearInterval(connectionsHealthCheck)
      }
      
      // 设置新的定时器，每5秒检查一次
      connectionsHealthCheck = window.setInterval(() => {
        const lastConnection = connections.value.length > 0 
          ? connections.value[connections.value.length - 1] 
          : null
        
        // 如果超过15秒没有新数据且状态为已连接，尝试重新连接
        if (connectionsState.value.connected && 
            (!lastConnection || Date.now() - new Date(lastConnection.start).getTime() > 15000)) {
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
        if (memoryState.value.connected && 
            Date.now() - memory.value.lastUpdated > 10000) {
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

    // 初始化连接监听
    const setupConnectionsListener = async () => {
      try {
        // 先清理可能存在的旧监听器
        if (unlistenConnectionsStateFn) {
          unlistenConnectionsStateFn()
          unlistenConnectionsStateFn = null
        }
        
        if (unlistenConnectionsDataFn) {
          unlistenConnectionsDataFn()
          unlistenConnectionsDataFn = null
        }
        
        // 设置状态事件监听
        unlistenConnectionsStateFn = await listen<ConnectionState>('connections-connection-state', (event) => {
          connectionsState.value = event.payload
          
          // 如果状态变为已连接，启动健康检查
          if (event.payload.connected) {
            startConnectionsHealthCheck()
          }
        })
        
        // 设置数据事件监听
        unlistenConnectionsDataFn = await listen<ConnectionsData>('connections-data', (event) => {
          const data = event.payload
          if (data && 'connections' in data) {
            connections.value = data.connections || []
            connectionsTotal.value = {
              upload: data.uploadTotal || 0,
              download: data.downloadTotal || 0,
            }
          }
        })
        
        // 尝试连接 WebSocket
        const connected = await wsService.connect('connections')
        
        if (connected) {
          console.log('连接 WebSocket 连接成功')
          // 连接成功后启动健康检查
          startConnectionsHealthCheck()
          return true
        } else {
          console.error('连接 WebSocket 连接失败')
          // 连接失败后，设置重试
          setTimeout(() => reconnectConnectionsWebSocket(), 3000)
          return false
        }
      } catch (error) {
        console.error('设置连接监听器最终失败:', error)
        // 异常情况下，也设置延迟重试
        setTimeout(() => reconnectConnectionsWebSocket(), 3000)
        return false
      }
    }

    // 初始化内存监听
    const setupMemoryListener = async () => {
      try {
        // 先清理可能存在的旧监听器
        if (unlistenMemoryStateFn) {
          unlistenMemoryStateFn()
          unlistenMemoryStateFn = null
        }
        
        if (unlistenMemoryDataFn) {
          unlistenMemoryDataFn()
          unlistenMemoryDataFn = null
        }
        
        // 设置状态事件监听
        unlistenMemoryStateFn = await listen<ConnectionState>('memory-connection-state', (event) => {
          memoryState.value = event.payload
          
          // 如果状态变为已连接，启动健康检查
          if (event.payload.connected) {
            startMemoryHealthCheck()
          }
        })
        
        // 设置数据事件监听
        unlistenMemoryDataFn = await listen('memory-data', (event) => {
          const data = event.payload as {
            inuse: number
            oslimit: number
          }
          if ('inuse' in data && 'oslimit' in data) {
            memory.value = {
              ...data,
              lastUpdated: Date.now() // 更新时间戳
            }
          }
        })
        
        // 尝试连接 WebSocket
        const connected = await wsService.connect('memory')
        
        if (connected) {
          console.log('内存 WebSocket 连接成功')
          // 连接成功后启动健康检查
          startMemoryHealthCheck()
          return true
        } else {
          console.error('内存 WebSocket 连接失败')
          // 连接失败后，设置重试
          setTimeout(() => reconnectMemoryWebSocket(), 3000)
          return false
        }
      } catch (error) {
        console.error('设置内存监听器最终失败:', error)
        // 异常情况下，也设置延迟重试
        setTimeout(() => reconnectMemoryWebSocket(), 3000)
        return false
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
        lastUpdated: Date.now()
      }
    }

    // 清理所有监听器
    const cleanupListeners = () => {
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
      
      // 断开 WebSocket 连接
      wsService.disconnect('connections').catch(e => console.error('断开连接 WebSocket 失败:', e))
      wsService.disconnect('memory').catch(e => console.error('断开内存 WebSocket 失败:', e))
    }

    // 更新连接数据
    const updateConnections = (data: ConnectionsData) => {
      if (data && 'connections' in data) {
        connections.value = data.connections || []
        connectionsTotal.value = {
          upload: data.uploadTotal || 0,
          download: data.downloadTotal || 0,
        }
      }
    }

    // 更新内存数据
    const updateMemory = (data: { inuse: number; oslimit: number }) => {
      if ('inuse' in data && 'oslimit' in data) {
        memory.value = {
          ...data,
          lastUpdated: Date.now() // 更新时间戳
        }
      }
    }

    return {
      connections,
      connectionsTotal,
      memory,
      connectionsState,
      memoryState,
      setupConnectionsListener,
      setupMemoryListener,
      cleanupListeners,
      resetData,
      updateConnections,
      updateMemory,
      reconnectConnectionsWebSocket,
      reconnectMemoryWebSocket
    }
  }
)
