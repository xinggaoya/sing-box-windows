import { defineStore } from 'pinia'
import { ref, onUnmounted } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { createWebSocket } from '@/utils'

// 定义版本信息接口
interface VersionInfo {
  version: string
  meta: boolean
  premium: boolean
  environment?: string
  tags?: string[]
  revision?: string
  cgo?: string
}

export const useInfoStore = defineStore(
  'info',
  () => {
    // 版本信息
    const version = ref<VersionInfo>({ version: '', meta: true, premium: true })
    const newVersion = ref('')

    // 内存使用信息
    const memory = ref({
      inuse: 0,
      oslimit: 0,
    })

    // 流量信息
    const traffic = ref({
      up: 0,
      down: 0,
      total: 0,
    })

    // 日志信息
    // 减少最大日志数量以减轻内存压力
    const MAX_LOGS = 200

    interface LogEntry {
      type: string
      payload: string
      timestamp: number
    }

    const logs = ref<LogEntry[]>([])

    // 存储 WebSocket 清理函数
    let cleanupFunctions: Array<() => void> = []
    // 记录是否存在活跃的WebSocket连接
    let activeConnections = false

    // 获取最新版本
    const getLatestVersion = async () => {
      try {
        const res = await fetch('https://api.github.com/repos/SagerNet/sing-box/releases/latest')
        const json = await res.json()
        newVersion.value = json.tag_name
      } catch (error) {
        console.error('获取最新版本失败:', error)
      }
    }

    // 检查内核版本
    const checkKernelVersion = async () => {
      try {
        const output = await tauriApi.kernel.checkKernelVersion()
        if (output) {
          const versionInfo: VersionInfo = {
            version: '',
            meta: true,
            premium: true,
          }

          // 解析版本输出
          const lines = output.split('\n')
          for (const line of lines) {
            if (line.startsWith('sing-box version')) {
              versionInfo.version = line.split(' ')[2]
            } else if (line.startsWith('Environment:')) {
              versionInfo.environment = line.split(':')[1].trim()
            } else if (line.startsWith('Tags:')) {
              versionInfo.tags = line.split(':')[1].trim().split(',')
            } else if (line.startsWith('Revision:')) {
              versionInfo.revision = line.split(':')[1].trim()
            } else if (line.startsWith('CGO:')) {
              versionInfo.cgo = line.split(':')[1].trim()
            }
          }

          version.value = versionInfo
          return true
        }
        return false
      } catch (error) {
        console.error('检查内核版本失败:', error)
        return false
      }
    }

    // 初始化 WebSocket 连接
    const initWebSocket = () => {
      // 如果已经有活跃的连接，先清理
      if (activeConnections) {
        cleanupWebSockets()
      }

      // 设置活跃标志
      activeConnections = true

      // 流量监控
      const cleanupTraffic = createWebSocket('ws://127.0.0.1:9090/traffic?token=', (data) => {
        if ('up' in data && 'down' in data) {
          const currentUp = Number(data.up) || 0
          const currentDown = Number(data.down) || 0
          traffic.value = {
            up: currentUp,
            down: currentDown,
            total: traffic.value.total + currentUp + currentDown,
          }
        }
      })

      // 内存监控
      const cleanupMemory = createWebSocket('ws://127.0.0.1:9090/memory?token=', (data) => {
        if ('inuse' in data && 'oslimit' in data) {
          memory.value = data
        }
      })

      // 日志监控
      const cleanupLogs = createWebSocket('ws://127.0.0.1:9090/logs?token=', (data) => {
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

      // 过滤掉null值，存储清理函数
      cleanupFunctions = [cleanupTraffic, cleanupMemory, cleanupLogs].filter(Boolean) as Array<
        () => void
      >
    }

    // 清理所有 WebSocket 连接
    const cleanupWebSockets = () => {
      if (cleanupFunctions.length > 0) {
        cleanupFunctions.forEach((cleanup) => cleanup())
        cleanupFunctions = []
        activeConnections = false
      }
    }

    // 启动内核
    const startKernel = async () => {
      await tauriApi.kernel.startKernel()

      // 等待内核启动并检查状态
      return new Promise((resolve, reject) => {
        let retryCount = 0
        const maxRetries = 5

        const checkStatus = async () => {
          try {
            const res = await fetch('http://127.0.0.1:9090/version')
            if (!res.ok) {
              throw new Error(`HTTP error! status: ${res.status}`)
            }

            const json = await res.json()
            version.value = json

            // 获取最新版本信息
            await getLatestVersion()

            // 初始化 WebSocket 连接
            initWebSocket()

            resolve(true)
          } catch (error) {
            console.error('检查状态失败:', error)
            if (retryCount < maxRetries) {
              retryCount++
              console.log(`重试第 ${retryCount} 次，共 ${maxRetries} 次`)
              setTimeout(checkStatus, 1000)
            } else {
              reject(new Error(`启动失败，已重试 ${maxRetries} 次: ${error}`))
            }
          }
        }

        checkStatus()
      })
    }

    // 停止内核
    const stopKernel = async () => {
      try {
        await tauriApi.kernel.stopKernel()
      } finally {
        // 无论成功与否，都清理WebSocket连接和状态
        cleanupWebSockets()
        // 重置状态
        memory.value = { inuse: 0, oslimit: 0 }
        traffic.value = { up: 0, down: 0, total: 0 }
        logs.value = []
      }
    }

    // 重启内核
    const restartKernel = async () => {
      await stopKernel()
      await startKernel()
    }

    // 更新版本信息
    const updateVersion = async () => {
      try {
        await checkKernelVersion()
      } catch (error) {
        console.error('获取版本信息失败:', error)
        version.value = { version: '', meta: false, premium: false }
      }
    }

    // 清理日志
    const clearLogs = () => {
      logs.value = []
    }

    // 组件卸载时清理
    onUnmounted(() => {
      cleanupWebSockets()
    })

    return {
      version,
      newVersion,
      memory,
      traffic,
      logs,
      startKernel,
      stopKernel,
      restartKernel,
      initWebSocket,
      updateVersion,
      checkKernelVersion,
      clearLogs,
      cleanupWebSockets,
    }
  },
  {
    persist: true,
  },
)
