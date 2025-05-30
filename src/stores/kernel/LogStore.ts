import { defineStore } from 'pinia'
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import mitt from '@/utils/mitt'
import { StoreCleaner } from '@/utils/memory-leak-fix'

// 定义消息类型
export type MessageType = 'success' | 'info' | 'error' | 'warning'

// 日志条目接口
export interface LogEntry {
  type: string
  payload: string
  timestamp: number
}

export const useLogStore = defineStore('log', () => {
  // 减少最大日志数量以减轻内存压力
  const MAX_LOGS = 500

  // 日志信息
  const logs = ref<LogEntry[]>([])

  // 消息回调函数
  let messageCallback: ((type: MessageType, content: string) => void) | null = null

  // 存储事件监听器清理函数
  let unlistenLogsFn: (() => void) | null = null

  // 是否已经设置了mitt监听器
  let mittListenerSet = false

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

  // 添加日志
  const addLog = (type: string, payload: string) => {
    // 添加新的日志条目
    logs.value.unshift({
      type,
      payload,
      timestamp: Date.now(),
    })

    // 如果超过最大日志数量，删除最旧的日志
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

  // 组件卸载时清理监听器
  onUnmounted(() => {
    cleanupListeners()
  })

  // 组件挂载时设置监听器（如果需要）
  onMounted(() => {
    // 注释掉自动设置，由调用者决定是否调用setupLogListener
    // setupLogListener()
  })

  return {
    logs,
    addLog,
    clearLogs,
    setMessageCallback,
    showMessage,
    setupLogListener,
    cleanupListeners,
  }
})
