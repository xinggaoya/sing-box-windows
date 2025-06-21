import { defineStore } from 'pinia'
import { ref, onMounted, onUnmounted } from 'vue'
import mitt from '@/utils/mitt'
import { StoreCleaner } from '@/utils/memory-leak-fix'

// 声明traffic-data事件的类型
interface TrafficData {
  up: number
  down: number
}

// 连接状态接口
interface ConnectionState {
  connected: boolean
  connecting: boolean
  error: Error | null
}

export const useTrafficStore = defineStore('traffic', () => {
  // 流量数据
  const traffic = ref({
    up: 0,
    down: 0,
    totalUp: 0,
    totalDown: 0,
  })

  // 连接状态
  const connectionState = ref<ConnectionState>({
    connected: false,
    connecting: false,
    error: null,
  })

  // 数据缓冲区，避免过度处理
  let dataBuffer: TrafficData[] = []
  let bufferProcessTimer: number | null = null

  // 事件监听器状态
  let mittListenerRegistered = false

  // 处理缓冲区数据
  const processBufferedData = () => {
    if (dataBuffer.length === 0) return

    // 取最新的数据
    const latestData = dataBuffer[dataBuffer.length - 1]

    // 更新流量数据
    if (latestData.up !== undefined && latestData.down !== undefined) {
      const upDiff = Math.max(0, latestData.up - traffic.value.up)
      const downDiff = Math.max(0, latestData.down - traffic.value.down)

      traffic.value.up = latestData.up
      traffic.value.down = latestData.down
      traffic.value.totalUp += upDiff
      traffic.value.totalDown += downDiff
    }

    // 清空缓冲区
    dataBuffer = []
  }

  // 启动缓冲区处理定时器
  const startBufferProcessor = () => {
    if (bufferProcessTimer) {
      clearInterval(bufferProcessTimer)
    }

    bufferProcessTimer = window.setInterval(() => {
      processBufferedData()
    }, 1000) // 每秒处理一次
  }

  // 停止缓冲区处理定时器
  const stopBufferProcessor = () => {
    if (bufferProcessTimer) {
      clearInterval(bufferProcessTimer)
      bufferProcessTimer = null
    }
  }

  // 更新流量统计数据
  const updateTrafficStats = (data: TrafficData) => {
    if (data && 'up' in data && 'down' in data) {
      try {
        // 将数据添加到缓冲区而不是立即处理
        dataBuffer.push(data)

        // 限制缓冲区大小
        if (dataBuffer.length > 10) {
          dataBuffer = dataBuffer.slice(-5)
        }

        // 如果数据接收正常，但当前状态不是连接状态，更新状态
        if (!connectionState.value.connected) {
          connectionState.value.connected = true
          connectionState.value.connecting = false
          connectionState.value.error = null
        }
      } catch (error) {
        console.error('处理流量数据时出错:', error, data)
      }
    }
  }

  // 设置Mitt事件监听器
  const setupMittListeners = () => {
    if (mittListenerRegistered) return

    // 监听流量数据事件
    mitt.on('traffic-data', (data) => {
      if (data && typeof data === 'object' && 'up' in data && 'down' in data) {
        updateTrafficStats(data as unknown as TrafficData)
      }
    })

    // 监听WebSocket连接状态
    mitt.on('ws-connected', () => {
      connectionState.value.connected = true
      connectionState.value.connecting = false
      connectionState.value.error = null
    })

    mitt.on('ws-disconnected', () => {
      connectionState.value.connected = false
      connectionState.value.connecting = false
    })

    mittListenerRegistered = true
    console.log('🔗 TrafficStore Mitt监听器已注册')
  }

  // 清理Mitt监听器
  const cleanupMittListeners = () => {
    if (!mittListenerRegistered) return

    mitt.off('traffic-data')
    mitt.off('ws-connected')
    mitt.off('ws-disconnected')

    mittListenerRegistered = false
    console.log('🧹 TrafficStore Mitt监听器已清理')
  }

  // 重新连接WebSocket
  const reconnectWebSocket = async () => {
    try {
      // 使用事件通知WebSocketService重连
      mitt.emit('websocket-reconnect', 'traffic')
    } catch (error) {
      console.error('重新连接流量WebSocket失败:', error)
    }
  }

  // 重置流量统计
  const resetStats = () => {
    traffic.value = {
      up: 0,
      down: 0,
      totalUp: 0,
      totalDown: 0,
    }
    connectionState.value = {
      connected: false,
      connecting: false,
      error: null,
    }
    dataBuffer = []
  }

  // 清理所有监听器
  const cleanupListeners = () => {
    // 清理Mitt监听器
    cleanupMittListeners()

    // 停止缓冲区处理
    stopBufferProcessor()

    console.log('🧹 TrafficStore 监听器已清理')
  }

  // 监听内存清理请求
  mitt.on('memory-cleanup-requested', () => {
    console.log('🧹 响应内存清理请求 - Traffic Store')

    // 清空大型数据结构
    dataBuffer = []

    // 重置流量统计以释放内存
    if (traffic.value.totalUp > 1024 * 1024 * 1024) {
      // 如果总流量超过1GB，重置统计
      resetStats()
      console.log('🧹 重置大流量统计数据')
    }
  })

  // 注册清理函数
  StoreCleaner.registerCleanup(() => {
    cleanupListeners()
    dataBuffer = []
  })

  // 组件挂载时初始化
  onMounted(() => {
    setupMittListeners()
    startBufferProcessor()
  })

  // 组件卸载时清理
  onUnmounted(() => {
    cleanupListeners()
  })

  return {
    traffic,
    connectionState,
    setupMittListeners,
    cleanupMittListeners,
    resetStats,
    cleanupListeners,
    updateTrafficStats,
    reconnectWebSocket,
  }
})
