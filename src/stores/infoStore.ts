import { defineStore } from 'pinia'
import { ref, onUnmounted } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { listen } from '@tauri-apps/api/event'

// 定义消息类型
export type MessageType = 'success' | 'info' | 'error' | 'warning'

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
      totalUp: 0, // 上传总流量
      totalDown: 0, // 下载总流量
    })

    // 连接信息
    const connections = ref<Connection[]>([])
    const connectionsTotal = ref({
      upload: 0,
      download: 0,
    })

    // 程序运行时间（秒）
    const uptime = ref(0)
    let uptimeInterval: NodeJS.Timeout | null = null

    // 日志信息
    // 减少最大日志数量以减轻内存压力
    const MAX_LOGS = 200

    interface LogEntry {
      type: string
      payload: string
      timestamp: number
    }

    const logs = ref<LogEntry[]>([])

    // 存储事件监听器清理函数
    let cleanupFunctions: Array<() => void> = []
    // 记录是否存在活跃的事件监听器
    let activeConnections = false

    // 获取最新版本
    const getLatestVersion = async () => {
      try {
        const res = await fetch('https://api.github.com/repos/SagerNet/sing-box/releases/latest')
        const json = await res.json()
        newVersion.value = json.tag_name.replace('v', '')
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

    // 初始化事件监听器
    const initEventListeners = async () => {
      // 如果已经有活跃的连接，先清理
      if (activeConnections) {
        cleanupEventListeners()
      }

      // 设置活跃标志
      activeConnections = true

      // 开始计算运行时间
      uptime.value = 0
      uptimeInterval = setInterval(() => {
        uptime.value += 1
      }, 1000)

      try {
        // 启动后端的WebSocket中继
        await tauriApi.kernel.startWebsocketRelay()

        // 监听流量数据
        const unlistenTraffic = await listen('traffic-data', (event) => {
          const data = event.payload as {
            up: number
            down: number
          }
          if ('up' in data && 'down' in data) {
            const currentUp = Number(data.up) || 0
            const currentDown = Number(data.down) || 0

            // 安全地更新总流量计数
            const currentTotalUp = Number(traffic.value.totalUp) || 0
            const currentTotalDown = Number(traffic.value.totalDown) || 0

            traffic.value = {
              up: currentUp,
              down: currentDown,
              total: traffic.value.total + currentUp + currentDown,
              totalUp: currentTotalUp + currentUp,
              totalDown: currentTotalDown + currentDown,
            }
          }
        })

        // 监听内存数据
        const unlistenMemory = await listen('memory-data', (event) => {
          const data = event.payload as {
            inuse: number
            oslimit: number
          }
          if ('inuse' in data && 'oslimit' in data) {
            memory.value = data
          }
        })

        // 监听日志数据
        const unlistenLogs = await listen('log-data', (event) => {
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

        // 监听连接数据
        const unlistenConnections = await listen('connections-data', (event) => {
          const data = event.payload as ConnectionsData
          if ('connections' in data && Array.isArray(data.connections)) {
            connections.value = data.connections

            // 更新总流量数据
            if ('downloadTotal' in data && 'uploadTotal' in data) {
              connectionsTotal.value = {
                download: data.downloadTotal || 0,
                upload: data.uploadTotal || 0,
              }
            }
          }
        })

        // 存储清理函数
        cleanupFunctions = [unlistenTraffic, unlistenMemory, unlistenLogs, unlistenConnections]
      } catch (error) {
        console.error('初始化事件监听失败:', error)
      }
    }

    // 清理所有事件监听器
    const cleanupEventListeners = () => {
      if (cleanupFunctions.length > 0) {
        cleanupFunctions.forEach((cleanup) => cleanup())
        cleanupFunctions = []
        activeConnections = false
      }

      // 清理运行时间计时器
      if (uptimeInterval) {
        clearInterval(uptimeInterval)
        uptimeInterval = null
      }
    }

    // 启动内核
    const startKernel = async () => {
      await tauriApi.kernel.startKernel()

      // 确保初始化时重置所有计数器
      traffic.value = {
        up: 0,
        down: 0,
        total: 0,
        totalUp: 0,
        totalDown: 0,
      }
      uptime.value = 0
      connections.value = []
      connectionsTotal.value = { upload: 0, download: 0 }

      // 等待内核启动并检查状态
      return new Promise((resolve, reject) => {
        let retryCount = 0
        const maxRetries = 5

        const checkStatus = async () => {
          try {
            // 使用Tauri API获取版本信息
            const json = await tauriApi.proxy.getVersionInfo()
            version.value = json

            // 获取最新版本信息
            await getLatestVersion()

            // 初始化事件监听器
            await initEventListeners()

            resolve(true)
          } catch (error) {
            console.error('检查状态失败:', error)
            if (retryCount < maxRetries) {
              retryCount++
              console.log(`重试第 ${retryCount} 次，共 ${maxRetries} 次`)
              setTimeout(checkStatus, 1000)
            } else {
              // 在无法获取版本信息的情况下，使用默认值，不阻止程序运行
              console.warn('无法获取版本信息，使用默认值')
              version.value = { version: 'sing-box 未知版本', meta: true, premium: true }

              // 尽管无法获取版本信息，仍然初始化事件监听器
              try {
                await initEventListeners()
                resolve(true)
              } catch (initError) {
                console.error('初始化事件监听器失败:', initError)
                reject(new Error(`初始化失败: ${initError}`))
              }
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
        // 无论成功与否，都清理事件监听器和状态
        cleanupEventListeners()
        // 重置状态
        memory.value = { inuse: 0, oslimit: 0 }
        traffic.value = { up: 0, down: 0, total: 0, totalUp: 0, totalDown: 0 }
        uptime.value = 0
        logs.value = []
        connections.value = []
        connectionsTotal.value = { upload: 0, download: 0 }
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

    // 消息通知功能
    let messageCallback: ((type: MessageType, content: string) => void) | null = null

    const setMessageCallback = (callback: (type: MessageType, content: string) => void) => {
      messageCallback = callback
    }

    const showMessage = (type: MessageType, content: string) => {
      if (messageCallback) {
        messageCallback(type, content)
      } else {
        console.log(`[${type}] ${content}`)
      }
    }

    // IP版本切换
    const toggleIpVersion = async () => {
      // 实现IP版本切换逻辑
    }

    // 重置统计信息
    const resetStats = () => {
      traffic.value = {
        up: 0,
        down: 0,
        total: 0,
        totalUp: 0,
        totalDown: 0,
      }
      uptime.value = 0
    }

    // 组件卸载时清理
    onUnmounted(() => {
      cleanupEventListeners()
    })

    return {
      version,
      newVersion,
      memory,
      traffic,
      logs,
      uptime,
      connections,
      connectionsTotal,
      startKernel,
      stopKernel,
      restartKernel,
      initEventListeners,
      updateVersion,
      checkKernelVersion,
      clearLogs,
      cleanupEventListeners,
      // 消息通知
      setMessageCallback,
      showMessage,
      // 内核操作
      toggleIpVersion,
      resetStats,
    }
  },
  {
    persist: true,
  },
)
