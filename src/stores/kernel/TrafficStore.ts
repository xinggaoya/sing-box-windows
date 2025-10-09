import { defineStore } from 'pinia'
import { ref } from 'vue'
import { eventService } from '@/services/event-service'
import { temporaryStoreManager } from '@/utils/memory-leak-fix'

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
    let eventListenersSetup = false

    // 内存清理定时器
    let memoryCleanupTimer: number | null = null

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

    // 设置Tauri事件监听器
    const setupEventListeners = async () => {
      if (eventListenersSetup) return

      try {
        // 监听流量数据事件
        await eventService.onTrafficData((data) => {
          if (data && typeof data === 'object' && 'up' in data && 'down' in data) {
            updateTrafficStats(data as unknown as TrafficData)
          }
        })

        // 当收到流量数据时，说明连接正常
        connectionState.value.connected = true
        connectionState.value.connecting = false
        connectionState.value.error = null

        eventListenersSetup = true
        console.log('✅ 流量Store事件监听器设置完成')
      } catch (error) {
        console.error('❌ 流量Store事件监听器设置失败:', error)
      }
    }

    // 清理事件监听器
    const cleanupEventListeners = () => {
      if (!eventListenersSetup) return

      try {
        eventService.removeEventListener('traffic-data')
      } catch (error) {
        console.error('清理流量监听器时出错:', error)
      } finally {
        eventListenersSetup = false
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
      cleanupEventListeners()
    }

    // Store初始化方法
    const initializeStore = async () => {
      await setupEventListeners()
      startMemoryOptimization()

      // 注册到临时Store管理器
      const storeInstance = {
        cleanupStore,
        smartCleanup: () => {
          // 如果累计流量超过500MB，重置计数器
          const RESET_THRESHOLD = 500 * 1024 * 1024 // 500MB
          if (
            traffic.value.totalUp > RESET_THRESHOLD ||
            traffic.value.totalDown > RESET_THRESHOLD
          ) {
            traffic.value.totalUp = 0
            traffic.value.totalDown = 0
            console.log('🧹 流量Store智能清理 - 重置累计数据')
          }
        },
      }
      temporaryStoreManager.registerStore('traffic', storeInstance)
    }

    // 内存优化：定期清理无用数据
    const startMemoryOptimization = () => {
      if (memoryCleanupTimer) {
        clearInterval(memoryCleanupTimer)
      }

      // 每30秒检查一次，重置累计流量如果数值过大
      memoryCleanupTimer = window.setInterval(() => {
        // 如果累计流量超过1GB，重置计数器防止数值溢出
        const MAX_TRAFFIC = 1024 * 1024 * 1024 // 1GB
        if (traffic.value.totalUp > MAX_TRAFFIC || traffic.value.totalDown > MAX_TRAFFIC) {
          traffic.value.totalUp = 0
          traffic.value.totalDown = 0
        }
      }, 30 * 1000) // 30秒
    }

    // 停止内存优化
    const stopMemoryOptimization = () => {
      if (memoryCleanupTimer) {
        clearInterval(memoryCleanupTimer)
        memoryCleanupTimer = null
      }
    }

    // Store清理方法
    const cleanupStore = () => {
      cleanupListeners()
      stopMemoryOptimization()
      resetStats()

      // 从临时Store管理器注销
      temporaryStoreManager.unregisterStore('traffic')
    }

    return {
      traffic,
      connectionState,
      setupEventListeners,
      cleanupEventListeners,
      cleanupListeners,
      resetStats,
      updateTrafficStats,
      startMemoryOptimization,
      stopMemoryOptimization,
      initializeStore,
      cleanupStore,
    }
  },
)