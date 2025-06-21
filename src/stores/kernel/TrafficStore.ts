import { defineStore } from 'pinia'
import { ref } from 'vue'
import mitt from '@/utils/mitt'

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

export const useTrafficStore = defineStore(
  'traffic',
  () => {
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

    // 事件监听器状态
    let mittListenerRegistered = false

    // 更新流量统计数据
    const updateTrafficStats = (data: TrafficData) => {
      if (data && 'up' in data && 'down' in data) {
        try {
          // 确保数据是数字类型
          const currentUp = Number(data.up) || 0
          const currentDown = Number(data.down) || 0
          const prevUp = Number(traffic.value.up) || 0
          const prevDown = Number(traffic.value.down) || 0

          const upDiff = Math.max(0, currentUp - prevUp)
          const downDiff = Math.max(0, currentDown - prevDown)

          // 直接更新数据，确保响应式更新
          traffic.value = {
            up: currentUp,
            down: currentDown,
            totalUp: (traffic.value.totalUp || 0) + upDiff,
            totalDown: (traffic.value.totalDown || 0) + downDiff,
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
    }

    // 清理Mitt监听器
    const cleanupMittListeners = () => {
      if (!mittListenerRegistered) return

      mitt.off('traffic-data')
      mitt.off('ws-connected')
      mitt.off('ws-disconnected')

      mittListenerRegistered = false
    }

    // 重新连接WebSocket
    const reconnectWebSocket = async () => {
      try {
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
    }

    // 清理所有监听器
    const cleanupListeners = () => {
      cleanupMittListeners()
    }

    return {
      traffic,
      connectionState,
      setupMittListeners,
      setupTrafficListener: setupMittListeners, // 为兼容性添加别名
      cleanupMittListeners,
      cleanupListeners,
      resetStats,
      updateTrafficStats,
      reconnectWebSocket,
    }
  },
  {
    // 恢复持久化配置 - 针对高频更新优化
    persist: {
      highFrequency: true, // 启用防抖保存
      debounceDelay: 3000, // 3秒防抖延迟
      excludeKeys: ['connectionState'], // 排除连接状态
    },
  },
)
