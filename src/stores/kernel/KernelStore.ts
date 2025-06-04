import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import mitt from '@/utils/mitt'
import { useAppStore } from '../app/AppStore'
import { useConnectionStore } from './ConnectionStore'
import { useTrafficStore } from './TrafficStore'
import { useLogStore } from './LogStore'
import { WebSocketService } from '@/services/websocket-service'

// 定义版本信息接口
export interface VersionInfo {
  version: string
  meta: boolean
  premium: boolean
  environment?: string
  tags?: string[]
  revision?: string
  cgo?: string
}

export const useKernelStore = defineStore(
  'kernel',
  () => {
    // 应用状态
    const appStore = useAppStore()

    // 版本信息
    const version = ref<VersionInfo>({ version: '', meta: true, premium: true })
    const newVersion = ref('')

    // 内存使用信息
    const memory = ref({
      inuse: 0,
      oslimit: 0,
    })

    // 程序运行时间（秒）
    const uptime = ref(0)
    let uptimeInterval: NodeJS.Timeout | null = null

    // 清理所有定时器
    const clearTimers = () => {
      if (uptimeInterval) {
        clearInterval(uptimeInterval)
        uptimeInterval = null
      }
    }

    // 更新版本信息
    const updateVersion = async () => {
      try {
        const versionString = await tauriApi.kernel.checkKernelVersion()
        if (versionString) {
          // 解析版本字符串为VersionInfo对象
          version.value = {
            version: versionString,
            meta: true,
            premium: true,
          }
          return true
        }
        return false
      } catch (error) {
        console.error('获取内核版本失败:', error)
        return false
      }
    }

    // 检查内核版本
    const checkKernelVersion = async () => {
      try {
        const versionInfo = await tauriApi.kernel.checkKernelVersion()
        if (versionInfo) {
          newVersion.value = versionInfo
          return true
        }
        return false
      } catch (error) {
        console.error('检查内核版本失败:', error)
        return false
      }
    }

    // 启动内核
    const startKernel = async () => {
      try {
        // 初始化运行时间计数器
        uptime.value = 0
        uptimeInterval = setInterval(() => {
          uptime.value += 1
        }, 1000)

        // 获取当前代理模式
        const proxyMode = appStore.proxyMode || 'manual'

        // 启动内核
        await tauriApi.kernel.startKernel(proxyMode)

        // 设置 WebSocket 连接 token
        const token = await tauriApi.proxy.getApiToken()
        const wsService = WebSocketService.getInstance()
        wsService.setToken(token)

        // 启动时将状态设为连接中
        appStore.setConnectingState(true)

        // 设置WebSocket连接检查
        return await checkWebSocketConnections(wsService)
      } catch (error) {
        // 停止计时器
        clearTimers()

        // 重置连接状态
        appStore.setConnectingState(false)
        appStore.setRunningState(false)

        // 格式化错误消息
        let errorMessage = '启动内核失败'
        if (error instanceof Error) {
          errorMessage = error.message
        } else if (typeof error === 'string') {
          errorMessage = error
        }

        console.error('启动内核失败:', errorMessage)

        // 通知启动失败
        mitt.emit('kernel-start-failed', { error: errorMessage })

        throw new Error(errorMessage)
      }
    }

    // 定时检查WebSocket连接状态
    const checkWebSocketConnections = async (wsService: WebSocketService) => {
      // 连接检查配置
      const maxCheckTime = 30000 // 最大检查时间（毫秒）
      const checkInterval = 1000 // 检查间隔（毫秒）
      const maxChecks = Math.floor(maxCheckTime / checkInterval)

      // 每次检查前清理可能存在的连接
      await wsService.disconnectAll().catch(() => {})

      // 开始定时检查
      let isConnected = false
      for (let i = 0; i < maxChecks; i++) {
        console.log(`检查WebSocket连接状态 (第${i + 1}/${maxChecks}次)...`)

        try {
          // 尝试建立连接
          isConnected = await wsService.checkAllConnections()

          if (isConnected) {
            console.log(`WebSocket连接成功 (第${i + 1}次检查)`)
            break
          } else {
            console.log(`WebSocket连接尚未就绪，${checkInterval}毫秒后重试...`)

            // 等待指定时间后重试
            await new Promise((resolve) => setTimeout(resolve, checkInterval))
          }
        } catch (error) {
          console.error(`WebSocket连接检查出错 (第${i + 1}次): ${error}`)

          // 等待后重试
          await new Promise((resolve) => setTimeout(resolve, checkInterval))
        }
      }

      if (!isConnected) {
        // 所有检查都失败，尝试停止内核并报错
        console.error(`WebSocket连接在${maxCheckTime / 1000}秒内检查失败，内核可能未正常启动`)

        // 清理资源
        clearTimers()
        await wsService.disconnectAll().catch(() => {})
        await tauriApi.kernel.stopKernel().catch(() => {})

        // 重置连接状态
        appStore.setConnectingState(false)

        // 抛出错误
        throw new Error(
          `启动失败: 内核服务在${maxCheckTime / 1000}秒内未就绪，请检查配置或网络问题`,
        )
      }

      // 成功建立WebSocket连接，设置运行状态
      appStore.setRunningState(true)
      appStore.setConnectingState(false)

      // 通知内核状态变更
      mitt.emit('kernel-started')

      return true
    }

    // 停止内核
    const stopKernel = async () => {
      try {
        // 清理计时器和事件监听器
        clearTimers()
        cleanupEventListeners()

        // 断开所有 WebSocket 连接
        const wsService = WebSocketService.getInstance()
        await wsService.disconnectAll()

        // 停止内核
        await tauriApi.kernel.stopKernel()

        // 设置运行状态
        appStore.setRunningState(false)

        // 重置所有相关数据
        const connectionStore = useConnectionStore()
        const trafficStore = useTrafficStore()

        // 重置内存使用信息
        memory.value = { inuse: 0, oslimit: 0 }

        // 重置数据
        connectionStore.resetData()
        trafficStore.resetStats()

        // 通知内核状态变更
        mitt.emit('kernel-stopped')

        return true
      } catch (error) {
        console.error('停止内核失败:', error)
        return false
      }
    }

    // 重启内核
    const restartKernel = async () => {
      try {
        // 先停止
        await stopKernel()

        // 短暂延迟确保完全停止
        await new Promise((resolve) => setTimeout(resolve, 500))

        // 再启动
        return await startKernel()
      } catch (error) {
        console.error('重启内核失败:', error)
        return false
      }
    }

    // 切换IP版本
    const toggleIpVersion = async (useIpv6: boolean) => {
      try {
        // 如果内核正在运行，需要重启
        const needRestart = appStore.isRunning

        if (needRestart) {
          await stopKernel()
        }

        // 更新IP版本设置
        appStore.preferIpv6 = useIpv6

        // 如果之前在运行，则重新启动
        if (needRestart) {
          await startKernel()
        }

        return true
      } catch (error) {
        console.error('切换IP版本失败:', error)
        return false
      }
    }

    // 初始化事件监听器
    const initEventListeners = async () => {
      try {
        // 更新版本信息
        await updateVersion()

        // 检查是否有新版本
        await checkKernelVersion()

        // 初始化运行时间计数器
        if (appStore.isRunning && !uptimeInterval) {
          uptime.value = 0
          uptimeInterval = setInterval(() => {
            uptime.value += 1
          }, 1000)
        }

        // 初始化连接监听器
        const connectionStore = useConnectionStore()
        await connectionStore.setupConnectionsListener()
        await connectionStore.setupMemoryListener()

        // 初始化流量监听器
        const trafficStore = useTrafficStore()
        await trafficStore.setupTrafficListener()

        // 初始化日志监听器
        const logStore = useLogStore()
        await logStore.setupLogListener()

        return true
      } catch (error) {
        console.error('初始化事件监听器失败:', error)
        return false
      }
    }

    // 清理事件监听器
    const cleanupEventListeners = () => {
      // 清理计时器
      clearTimers()

      // 清理连接监听器
      const connectionStore = useConnectionStore()
      connectionStore.cleanupListeners()

      // 清理流量监听器
      const trafficStore = useTrafficStore()
      trafficStore.cleanupListeners()

      // 清理日志监听器
      const logStore = useLogStore()
      logStore.cleanupListeners()
    }

    // 设置WebSocket连接
    const setupWebsocketConnection = async () => {
      try {
        // 启动WebSocket数据中继
        await tauriApi.kernel.startWebSocketRelay()

        // 设置WebSocket连接检查
        const wsService = WebSocketService.getInstance()
        return await checkWebSocketConnections(wsService)
      } catch (error) {
        console.error('设置WebSocket连接失败:', error)
        throw error
      }
    }

    return {
      version,
      newVersion,
      memory,
      uptime,
      updateVersion,
      checkKernelVersion,
      startKernel,
      stopKernel,
      restartKernel,
      setupWebsocketConnection,
      toggleIpVersion,
      initEventListeners,
      cleanupEventListeners,
    }
  },
  {
    persist: true,
  },
)
