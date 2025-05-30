import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { WebSocketService, ConnectionState } from '@/services/websocket-service'
import mitt from '@/utils/mitt'
import { webSocketCleaner, StoreCleaner } from '@/utils/memory-leak-fix'

// 声明traffic-data事件的类型
interface TrafficData {
  up: number
  down: number
}

// 流量存储接口
interface TrafficStorage {
  up: number
  down: number
  total: number
  totalUp: number
  totalDown: number
  lastUpdated: number // 添加最后更新时间戳
}

export const useTrafficStore = defineStore('traffic', () => {
  // WebSocket 服务实例
  const wsService = WebSocketService.getInstance()

  // 连接状态
  const connectionState = ref<ConnectionState>({
    connected: false,
    connecting: false,
    error: null,
  })

  // 流量信息
  const traffic = ref<TrafficStorage>({
    up: 0,
    down: 0,
    total: 0,
    totalUp: 0, // 上传总流量
    totalDown: 0, // 下载总流量
    lastUpdated: Date.now(), // 初始化最后更新时间
  })

  // 存储事件监听器清理函数
  let unlistenTrafficStateFn: (() => void) | null = null
  let unlistenTrafficDataFn: (() => void) | null = null
  let healthCheckInterval: number | null = null // 健康检查定时器
  let bufferProcessInterval: number | null = null // 缓冲处理定时器
  let dataBuffer: TrafficData[] = [] // 数据缓冲区，避免频繁更新

  // 健康检查函数 - 如果长时间没有更新，尝试重连
  const startHealthCheck = () => {
    // 清除已有的定时器
    if (healthCheckInterval !== null) {
      clearInterval(healthCheckInterval)
      healthCheckInterval = null
    }

    // 设置新的定时器，每5秒检查一次
    healthCheckInterval = window.setInterval(() => {
      const now = Date.now()
      // 如果超过10秒没有更新数据且状态为已连接，尝试重新连接
      if (now - traffic.value.lastUpdated > 10000 && connectionState.value.connected) {
        console.log('流量数据超过10秒未更新，尝试重新连接...')
        reconnectWebSocket()
      }
    }, 5000) as unknown as number
  }

  // 重新连接WebSocket
  const reconnectWebSocket = async () => {
    try {
      // 断开现有连接
      await wsService.disconnect('traffic')
      // 短暂延迟后重新连接
      setTimeout(async () => {
        await wsService.connect('traffic')
      }, 1000)
    } catch (error) {
      console.error('重新连接流量WebSocket失败:', error)
    }
  }

  // 数据缓冲处理函数
  const processBufferedData = () => {
    if (dataBuffer.length === 0) return

    // 批量处理数据，减少频繁更新
    const latestData = dataBuffer[dataBuffer.length - 1]

    // 优化数据处理：确保是数值类型，避免格式错误
    const upValue = Number(latestData.up) || 0
    const downValue = Number(latestData.down) || 0

    // 更新当前速率
    traffic.value.up = upValue
    traffic.value.down = downValue

    // 更新总流量
    traffic.value.totalUp += upValue
    traffic.value.totalDown += downValue
    traffic.value.total = traffic.value.totalUp + traffic.value.totalDown

    // 更新最后更新时间
    traffic.value.lastUpdated = Date.now()

    // 清空缓冲区
    dataBuffer = []
  }

  // 初始化流量监听
  const setupTrafficListener = async () => {
    try {
      // 先清理可能存在的旧监听器
      if (unlistenTrafficStateFn) {
        unlistenTrafficStateFn()
        unlistenTrafficStateFn = null
      }

      if (unlistenTrafficDataFn) {
        unlistenTrafficDataFn()
        unlistenTrafficDataFn = null
      }

      // 设置状态事件监听
      unlistenTrafficStateFn = await listen<ConnectionState>(
        'traffic-connection-state',
        (event) => {
          connectionState.value = event.payload

          // 如果状态变为已连接，启动健康检查
          if (event.payload.connected) {
            startHealthCheck()
          }
        },
      )

      // 设置数据事件监听，使用缓冲机制
      unlistenTrafficDataFn = await listen<TrafficData>('traffic-data', (event) => {
        const data = event.payload
        if (data && 'up' in data && 'down' in data) {
          // 将数据添加到缓冲区而不是立即处理
          dataBuffer.push(data)

          // 限制缓冲区大小，避免内存累积
          if (dataBuffer.length > 10) {
            dataBuffer = dataBuffer.slice(-5) // 只保留最后5个
          }
        }
      })

      // 设置定时处理缓冲数据，减少更新频率
      bufferProcessInterval = window.setInterval(processBufferedData, 1000) // 每秒处理一次
      webSocketCleaner.registerTimer(bufferProcessInterval)

      // 尝试连接 WebSocket
      const connected = await wsService.connect('traffic')

      if (connected) {
        console.log('流量 WebSocket 连接成功')
        // 连接成功后启动健康检查
        startHealthCheck()
        return true
      } else {
        console.error('流量 WebSocket 连接失败')
        // 连接失败后，设置重试
        setTimeout(() => reconnectWebSocket(), 3000)
        return false
      }
    } catch (error) {
      console.error('设置流量监听器最终失败:', error)
      // 异常情况下，也设置延迟重试
      setTimeout(() => reconnectWebSocket(), 3000)
      return false
    }
  }

  // 重置流量统计
  const resetStats = () => {
    traffic.value.up = 0
    traffic.value.down = 0
    traffic.value.totalUp = 0
    traffic.value.totalDown = 0
    traffic.value.total = 0
    traffic.value.lastUpdated = Date.now()

    // 清空数据缓冲区
    dataBuffer = []
  }

  // 清理监听器
  const cleanupListeners = () => {
    console.log('🧹 开始清理流量Store监听器')

    if (unlistenTrafficStateFn) {
      unlistenTrafficStateFn()
      unlistenTrafficStateFn = null
    }

    if (unlistenTrafficDataFn) {
      unlistenTrafficDataFn()
      unlistenTrafficDataFn = null
    }

    // 清除健康检查定时器
    if (healthCheckInterval !== null) {
      clearInterval(healthCheckInterval)
      healthCheckInterval = null
    }

    // 清除缓冲处理定时器
    if (bufferProcessInterval !== null) {
      clearInterval(bufferProcessInterval)
      bufferProcessInterval = null
    }

    // 清空数据缓冲区
    dataBuffer = []

    // 断开 WebSocket 连接
    wsService.disconnect('traffic').catch((e) => console.error('断开流量 WebSocket 失败:', e))
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

  return {
    traffic,
    connectionState,
    setupTrafficListener,
    resetStats,
    cleanupListeners,
    updateTrafficStats,
    reconnectWebSocket,
  }
})
