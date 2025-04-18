import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'

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
    })

    // 存储事件监听器清理函数
    let unlistenConnectionsFn: (() => void) | null = null
    let unlistenMemoryFn: (() => void) | null = null

    // 初始化连接监听
    const setupConnectionsListener = async () => {
      try {
        // 先清理可能存在的旧监听器
        if (unlistenConnectionsFn) {
          unlistenConnectionsFn()
          unlistenConnectionsFn = null
        }
        
        // 监听连接数据
        unlistenConnectionsFn = await listen<ConnectionsData>('connections-data', (event) => {
          const data = event.payload
          if (data && 'connections' in data) {
            connections.value = data.connections || []
            connectionsTotal.value = {
              upload: data.uploadTotal || 0,
              download: data.downloadTotal || 0,
            }
          }
        })
        
        return true
      } catch (error) {
        console.error('设置连接监听器失败:', error)
        return false
      }
    }

    // 初始化内存监听
    const setupMemoryListener = async () => {
      try {
        // 先清理可能存在的旧监听器
        if (unlistenMemoryFn) {
          unlistenMemoryFn()
          unlistenMemoryFn = null
        }
        
        // 监听内存数据
        unlistenMemoryFn = await listen('memory-data', (event) => {
          const data = event.payload as {
            inuse: number
            oslimit: number
          }
          if ('inuse' in data && 'oslimit' in data) {
            memory.value = data
          }
        })
        
        return true
      } catch (error) {
        console.error('设置内存监听器失败:', error)
        return false
      }
    }

    // 清理所有监听器
    const cleanupListeners = () => {
      if (unlistenConnectionsFn) {
        unlistenConnectionsFn()
        unlistenConnectionsFn = null
      }
      
      if (unlistenMemoryFn) {
        unlistenMemoryFn()
        unlistenMemoryFn = null
      }
    }

    return {
      connections,
      connectionsTotal,
      memory,
      setupConnectionsListener,
      setupMemoryListener,
      cleanupListeners,
    }
  }
)
