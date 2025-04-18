import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'

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

    // 日志信息
    const logs = ref<LogEntry[]>([])
    
    // 消息回调函数
    let messageCallback: ((type: MessageType, content: string) => void) | null = null
    
    // 存储事件监听器清理函数
    let unlistenLogsFn: (() => void) | null = null

    // 初始化日志监听
    const setupLogListener = async () => {
      try {
        // 先清理可能存在的旧监听器
        if (unlistenLogsFn) {
          unlistenLogsFn()
          unlistenLogsFn = null
        }
        
        // 监听日志数据
        unlistenLogsFn = await listen('log-data', (event) => {
          const data = event.payload as {
            type: string
            payload: string
          }
          if (
            'type' in data &&
            'payload' in data &&
            typeof data.type === 'string' &&
            typeof data.payload === 'string'
          ) {
            // 日志条目添加到数组前端，并限制最大数量
            logs.value.unshift({
              type: data.type,
              payload: data.payload,
              timestamp: Date.now(),
            })

            // 超过最大数量时，移除多余日志
            if (logs.value.length > MAX_LOGS) {
              logs.value = logs.value.slice(0, MAX_LOGS)
            }
          }
        })
        
        return true
      } catch (error) {
        console.error('设置日志监听器失败:', error)
        return false
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
      if (unlistenLogsFn) {
        unlistenLogsFn()
        unlistenLogsFn = null
      }
    }

    return {
      logs,
      addLog,
      clearLogs,
      setMessageCallback,
      showMessage,
      setupLogListener,
      cleanupListeners
    }
  },
  {
    persist: true,
  }
)
