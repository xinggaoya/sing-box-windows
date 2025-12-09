import { defineStore } from 'pinia'
import { ref } from 'vue'
import { APP_EVENTS } from '@/constants/events'
import { eventService } from '@/services/event-service'
import type { LogEventPayload } from '@/types/events'

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

    // 事件监听器状态
    let eventListenersSetup = false

    // 日志清理定时器
    let logCleanupInterval: number | null = null

    // 内存监控定时器
    let memoryMonitorTimer: number | null = null

    // 初始化日志监听
    const setupLogListener = async () => {
      try {
        // 先清理可能存在的旧监听器
        cleanupListeners()

        // 监听日志数据事件
        await eventService.onLogData((data) => {
          processLogData(data)
        })

        // 启动定期清理机制
        startPeriodicCleanup()
        startMemoryMonitoring()

        eventListenersSetup = true
        console.log('✅ 日志Store事件监听器设置完成')
        return true
      } catch (error) {
        console.error('❌ 设置日志监听器失败:', error)
        return false
      }
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
          typeof (data as LogEventPayload).type === 'string' &&
          typeof (data as LogEventPayload).payload === 'string'
        ) {
          const entry = data as LogEventPayload
          addLog(entry.type, entry.payload)
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

      if (eventListenersSetup) {
        try {
          eventService.removeEventListener(APP_EVENTS.logData)
        } catch (error) {
          console.error('清理日志监听器时出错:', error)
        } finally {
          eventListenersSetup = false
        }
      }

      // 清理定期清理定时器
      if (logCleanupInterval) {
        clearInterval(logCleanupInterval)
        logCleanupInterval = null
      }

      stopMemoryMonitoring()
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

    // 初始化Store
    const initializeStore = async () => {
      try {
        console.log('🔧 初始化 LogStore...')
        await setupLogListener()
        console.log('✅ LogStore 初始化完成')
      } catch (error) {
        console.error('❌ LogStore 初始化失败:', error)
      }
    }

    return {
      logs,
      addLog,
      clearLogs,
      setMessageCallback,
      showMessage,
      setupLogListener,
      cleanupListeners,
      initializeStore, // 添加这个方法
    }
  },
)
