import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import mitt from '@/utils/mitt'
import { StoreCleaner, temporaryStoreManager } from '@/utils/memory-leak-fix'

// 定义消息类型
export type MessageType = 'success' | 'info' | 'error' | 'warning'

// 日志条目接口
export interface LogEntry {
  type: string
  payload: string
  timestamp: number
}

export const useLogStore = defineStore(
  'log',
  () => {
    // 减少最大日志数量以减轻内存压力
    const MAX_LOGS = 200
    // 设置内存警告阈值
    const MEMORY_WARNING_THRESHOLD = 150

    // 日志信息
    const logs = ref<LogEntry[]>([])

    // 消息回调函数
    let messageCallback: ((type: MessageType, content: string) => void) | null = null

    // 存储事件监听器清理函数
    let unlistenLogsFn: (() => void) | null = null

    // 是否已经设置了mitt监听器
    let mittListenerSet = false

    // 日志清理定时器
    let logCleanupInterval: number | null = null

    // 内存监控定时器
    let memoryMonitorTimer: number | null = null

    // 初始化日志监听
    const setupLogListener = async () => {
      try {
        // 先清理可能存在的旧监听器
        cleanupListeners()

        // 监听Tauri日志事件
        unlistenLogsFn = await listen('log-data', (event) => {
          processLogData(event.payload)
        })

        // 监听mitt事件总线的日志事件（从WebSocket服务中转发）
        if (!mittListenerSet) {
          mitt.on('log-data', handleMittLogData)
          mittListenerSet = true
        }

        // 启动定期清理机制
        startPeriodicCleanup()

        return true
      } catch (error) {
        console.error('设置日志监听器失败:', error)
        return false
      }
    }

    // 处理mitt事件总线上的日志数据
    const handleMittLogData = (data: unknown) => {
      processLogData(data)
    }

    // 处理日志数据
    const processLogData = (data: unknown) => {
      try {
        if (!data) {
          console.warn('日志数据为空')
          return
        }

        // 确保数据有正确的格式
        if (
          typeof data === 'object' &&
          data !== null &&
          'type' in data &&
          'payload' in data &&
          typeof data.type === 'string' &&
          typeof data.payload === 'string'
        ) {
          // 添加日志
          addLog(data.type, data.payload)
        } else {
          console.warn('无效的日志数据格式:', data)
        }
      } catch (e) {
        console.error('处理日志数据失败:', e)
      }
    }

    // 智能日志清理 - 基于时间和数量
    const smartLogCleanup = () => {
      const now = Date.now()
      const HOUR_MS = 60 * 60 * 1000 // 1小时

      // 移除1小时前的日志，但至少保留50条最新日志
      const recentLogs = logs.value.filter((log, index) => {
        const isRecent = now - log.timestamp < HOUR_MS
        const isInRecentRange = index < 50
        return isRecent || isInRecentRange
      })

      if (recentLogs.length < logs.value.length) {
        logs.value = recentLogs
        console.log(`🧹 智能清理日志，保留 ${recentLogs.length} 条`)
      }
    }

    // 启动内存监控
    const startMemoryMonitoring = () => {
      if (memoryMonitorTimer) {
        clearInterval(memoryMonitorTimer)
      }

      memoryMonitorTimer = window.setInterval(() => {
        // 如果日志数量接近警告阈值，执行智能清理
        if (logs.value.length >= MEMORY_WARNING_THRESHOLD) {
          smartLogCleanup()
        }
      }, 30 * 1000) // 30秒检查一次
    }

    // 停止内存监控
    const stopMemoryMonitoring = () => {
      if (memoryMonitorTimer) {
        clearInterval(memoryMonitorTimer)
        memoryMonitorTimer = null
      }
    }

    // 添加日志（优化版本）
    const addLog = (type: string, payload: string) => {
      // 防止重复日志（相同内容在10秒内不重复添加）
      const now = Date.now()
      const recentSimilarLog = logs.value.find(
        (log) => log.payload === payload && now - log.timestamp < 10000,
      )

      if (recentSimilarLog) {
        return // 跳过重复日志
      }

      // 添加新的日志条目
      logs.value.unshift({
        type,
        payload,
        timestamp: now,
      })

      // 立即清理如果超过最大数量
      if (logs.value.length > MAX_LOGS) {
        logs.value = logs.value.slice(0, MAX_LOGS)
      }
    }

    // 清空日志
    const clearLogs = () => {
      logs.value = []
      addLog('info', '日志已清空')
    }

    // 设置消息回调
    const setMessageCallback = (callback: (type: MessageType, content: string) => void) => {
      messageCallback = callback
    }

    // 显示消息
    const showMessage = (type: MessageType, content: string) => {
      // 记录到日志
      addLog(type, content)

      // 如果有回调，则调用回调
      if (messageCallback) {
        messageCallback(type, content)
      }
    }

    // 清理监听器
    const cleanupListeners = () => {
      console.log('🧹 开始清理日志Store监听器')

      if (unlistenLogsFn) {
        console.log('清理Tauri日志监听器')
        unlistenLogsFn()
        unlistenLogsFn = null
      }

      if (mittListenerSet) {
        console.log('清理mitt日志监听器')
        mitt.off('log-data', handleMittLogData)
        mittListenerSet = false
      }

      // 清理定期清理定时器
      if (logCleanupInterval) {
        clearInterval(logCleanupInterval)
        logCleanupInterval = null
      }
    }

    // 监听内存清理请求
    mitt.on('memory-cleanup-requested', () => {
      console.log('🧹 响应内存清理请求 - Log Store')

      // 如果日志过多，清理旧日志
      if (logs.value.length > MAX_LOGS / 2) {
        logs.value = logs.value.slice(0, MAX_LOGS / 2)
        console.log('🧹 清理了旧日志数据')
      }
    })

    // 注册清理函数
    StoreCleaner.registerCleanup(() => {
      cleanupListeners()
      logs.value = []
    })

    // Store初始化方法
    const initializeStore = () => {
      startMemoryMonitoring()

      // 注册到临时Store管理器
      const storeInstance = {
        cleanupStore,
        smartLogCleanup,
      }
      temporaryStoreManager.registerStore('log', storeInstance)
    }

    // Store清理方法
    const cleanupStore = () => {
      cleanupListeners()
      stopMemoryMonitoring()
      // 清空日志数据
      logs.value = []

      // 从临时Store管理器注销
      temporaryStoreManager.unregisterStore('log')
    }

    // 启动定期清理机制
    const startPeriodicCleanup = () => {
      if (logCleanupInterval) {
        clearInterval(logCleanupInterval)
      }

      // 每5分钟检查一次日志数量
      logCleanupInterval = window.setInterval(
        () => {
          if (logs.value.length > MAX_LOGS / 2) {
            // 只保留一半的日志
            logs.value = logs.value.slice(0, MAX_LOGS / 2)
            console.log('🧹 定期清理旧日志，当前保留', logs.value.length, '条')
          }
        },
        5 * 60 * 1000,
      ) // 5分钟
    }

    return {
      logs,
      addLog,
      clearLogs,
      setMessageCallback,
      showMessage,
      setupLogListener,
      cleanupListeners,
      smartLogCleanup,
      startMemoryMonitoring,
      stopMemoryMonitoring,
      initializeStore,
      cleanupStore,
    }
  },
  {
    // 日志数据不需要持久化存储 - 应用重启时重置日志
    persist: false,
  },
)
