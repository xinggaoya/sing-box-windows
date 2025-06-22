import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import mitt from '@/utils/mitt'
import { useAppStore } from '../app/AppStore'
import { useConnectionStore } from './ConnectionStore'
import { useTrafficStore } from './TrafficStore'
import { useLogStore } from './LogStore'
import { useKernelRuntimeStore } from './KernelRuntimeStore'
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

    // 版本信息 (需要持久化)
    const version = ref<VersionInfo>({ version: '', meta: true, premium: true })
    const newVersion = ref('')

    // 下载检查定时器
    let downloadCheckInterval: NodeJS.Timeout | null = null

    // 启动过程定时器
    let startupTimer: NodeJS.Timeout | null = null

    // 清理所有定时器
    const clearTimers = () => {
      if (downloadCheckInterval) {
        clearInterval(downloadCheckInterval)
        downloadCheckInterval = null
      }
      // 清理启动过程中的临时定时器
      if (startupTimer) {
        clearInterval(startupTimer)
        startupTimer = null
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

    // 启动内核（简化版本，主要依赖后端检查）
    const startKernel = async () => {
      console.log('🚀 开始启动内核...')

      try {
        // 获取当前代理模式
        const proxyMode = appStore.proxyMode || 'manual'

        // 设置连接中状态
        appStore.setConnectingState(true)
        console.log('📡 正在启动内核进程...')

        // 启动内核 - 后端会进行完整的就绪检查
        await tauriApi.kernel.startKernel(proxyMode)
        console.log('✅ 后端确认内核启动成功')

        // 后端检查通过，设置运行状态
        appStore.setRunningState(true)
        appStore.setConnectingState(false)

        // 后台初始化WebSocket连接（非阻塞）
        initializeWebSocketConnectionsAsync()

        // 通知内核状态变更
        mitt.emit('kernel-started')
        console.log('🎉 内核启动完成')

        return true
      } catch (error) {
        // 启动失败处理
        console.error('❌ 内核启动失败:', error)

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

        // 通知启动失败
        mitt.emit('kernel-start-failed', { error: errorMessage })

        throw new Error(errorMessage)
      }
    }

    // 定时检查WebSocket连接状态
    const checkWebSocketConnections = async (wsService: WebSocketService) => {
      // 连接检查配置 - 针对自动启动进行优化
      const maxCheckTime = 45000 // 增加到45秒最大检查时间
      const initialDelay = 3000 // 初始等待时间，给WebSocket服务更多启动时间
      const checkInterval = 2000 // 增加检查间隔到2秒
      const maxChecks = Math.floor((maxCheckTime - initialDelay) / checkInterval)

      console.log(`🔌 开始WebSocket连接检查，等待${initialDelay}ms后开始尝试连接...`)

      // 初始延迟，给内核的WebSocket服务更多启动时间
      await new Promise((resolve) => setTimeout(resolve, initialDelay))

      // 每次检查前清理可能存在的连接
      await wsService.disconnectAll().catch(() => {})

      // 开始定时检查
      let isConnected = false
      let lastError: Error | null = null

      for (let i = 0; i < maxChecks; i++) {
        console.log(`🔍 检查WebSocket连接状态 (第${i + 1}/${maxChecks}次)...`)

        try {
          // 尝试建立连接
          isConnected = await wsService.checkAllConnections()

          if (isConnected) {
            console.log(`✅ WebSocket连接成功 (第${i + 1}次检查)`)
            break
          } else {
            console.log(`⏳ WebSocket连接尚未就绪，${checkInterval}毫秒后重试...`)

            // 等待指定时间后重试
            await new Promise((resolve) => setTimeout(resolve, checkInterval))
          }
        } catch (error) {
          lastError = error instanceof Error ? error : new Error(String(error))
          console.error(`❌ WebSocket连接检查出错 (第${i + 1}次): ${error}`)

          // 等待后重试
          await new Promise((resolve) => setTimeout(resolve, checkInterval))
        }
      }

      if (!isConnected) {
        // 检查内核进程是否真的在运行
        console.log('🔍 WebSocket连接失败，检查内核进程状态...')

        try {
          const isKernelRunning = await tauriApi.kernel.isKernelRunning()
          console.log(`📊 内核进程状态: ${isKernelRunning ? '运行中' : '未运行'}`)

          if (isKernelRunning) {
            // 内核在运行但WebSocket连接失败，采用兼容模式
            console.warn('⚠️ 内核进程正在运行但WebSocket连接失败，启用兼容模式')

            // 设置运行状态但不设置WebSocket连接状态
            appStore.setRunningState(true)
            appStore.setConnectingState(false)

            // 安排后台重试WebSocket连接（仅重试一次，避免无限循环）
            setTimeout(() => {
              console.log('🔄 后台重试WebSocket连接（仅此一次）...')
              wsService
                .checkAllConnections()
                .then((success) => {
                  if (success) {
                    console.log('✅ 后台WebSocket连接成功')
                    mitt.emit('kernel-started')
                  } else {
                    console.log('⚠️ 后台WebSocket连接失败，但内核继续运行（不再重试）')
                  }
                })
                .catch((error) => {
                  console.error('❌ 后台WebSocket连接重试失败（不再重试）:', error)
                })
            }, 10000) // 延迟从5秒增加到10秒

            // 返回成功，允许应用继续运行
            mitt.emit('kernel-started')
            return true
          } else {
            // 内核进程确实没有运行
            throw new Error('内核进程启动失败')
          }
        } catch (checkError) {
          console.error('❌ 检查内核进程状态失败:', checkError)
          // 如果无法检查内核状态，还是抛出原始错误
        }

        // 所有检查都失败，尝试停止内核并报错
        console.error(`❌ WebSocket连接在${maxCheckTime / 1000}秒内检查失败，内核可能未正常启动`)
        console.error('最后一次错误:', lastError)

        // 清理资源
        clearTimers()
        await wsService.disconnectAll().catch(() => {})
        await tauriApi.kernel.stopKernel().catch(() => {})

        // 重置连接状态
        appStore.setConnectingState(false)

        // 抛出错误
        throw new Error(
          `启动失败: WebSocket服务在${maxCheckTime / 1000}秒内未就绪，请检查配置或网络问题`,
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
        const runtimeStore = useKernelRuntimeStore()

        // 重置数据
        connectionStore.resetData()
        trafficStore.resetStats()
        runtimeStore.resetRuntimeData()

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

        // 初始化连接监听器
        const connectionStore = useConnectionStore()
        connectionStore.setupMittListeners()

        // 初始化流量监听器
        const trafficStore = useTrafficStore()
        trafficStore.setupMittListeners()

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

    // 后台初始化WebSocket连接（非阻塞）
    const initializeWebSocketConnectionsAsync = async () => {
      console.log('🔌 后台初始化WebSocket连接...')

      try {
        // 获取API token
        const token = await tauriApi.proxy.getApiToken()
        const wsService = WebSocketService.getInstance()
        wsService.setToken(token)

        // 启动WebSocket数据中继
        await tauriApi.kernel.startWebSocketRelay()

        // 确保相关Store已初始化（自动启动时很重要）
        console.log('📦 确保相关Store已初始化...')
        await ensureDataStoresInitialized()

        // 尝试建立WebSocket连接
        const success = await wsService.checkAllConnections()

        if (success) {
          console.log('✅ WebSocket连接成功建立')
        } else {
          console.warn('⚠️ WebSocket连接建立失败，但内核继续运行')
        }
      } catch (error) {
        console.error('❌ WebSocket连接初始化失败:', error)
        // 不抛出错误，让内核继续运行
      }
    }

    // 确保数据相关的Store已初始化
    const ensureDataStoresInitialized = async () => {
      try {
        // 动态导入StoreManager避免循环依赖
        const { storeManager } = await import('../StoreManager')

        // 预加载所有数据相关的Store
        await storeManager.preloadStores(['connection', 'traffic', 'log'])
        console.log('📦 数据Store预加载完成')

        // 立即手动初始化这些Store的事件监听器，确保在WebSocket连接前就准备好
        try {
          const connectionStore = storeManager.getLoadedStore('connection')
          if (connectionStore) {
            // @ts-expect-error - Store类型推断问题，安全调用
            connectionStore.initializeStore?.()
            console.log('📡 ConnectionStore事件监听器已初始化')
          }
        } catch (error) {
          console.warn('ConnectionStore初始化警告:', error)
        }

        try {
          const trafficStore = storeManager.getLoadedStore('traffic')
          if (trafficStore) {
            // @ts-expect-error - Store类型推断问题，安全调用
            trafficStore.initializeStore?.()
            console.log('📊 TrafficStore事件监听器已初始化')
          }
        } catch (error) {
          console.warn('TrafficStore初始化警告:', error)
        }

        try {
          const logStore = storeManager.getLoadedStore('log')
          if (logStore) {
            // @ts-expect-error - Store类型推断问题，安全调用
            logStore.initializeStore?.()
            console.log('📝 LogStore事件监听器已初始化')
          }
        } catch (error) {
          console.warn('LogStore初始化警告:', error)
        }

        console.log('✅ 所有数据Store事件监听器初始化完成')
      } catch (error) {
        console.error('❌ Store初始化失败:', error)
      }
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

    // 重置临时数据 (应用启动时调用) - 现在委托给运行时store
    const resetTemporaryData = () => {
      // 获取运行时store并重置数据
      const runtimeStore = useKernelRuntimeStore()
      runtimeStore.resetRuntimeData()

      // 清理可能存在的定时器
      clearTimers()

      console.log('🔄 临时数据已重置')
    }

    // Store初始化方法
    const initializeStore = async () => {
      try {
        // 获取运行时store并初始化
        const runtimeStore = useKernelRuntimeStore()
        runtimeStore.initializeStore()

        // 如果应用正在运行，恢复运行时间计数器
        if (appStore.isRunning) {
          runtimeStore.startUptimeCounter()
          console.log('⏱️ 恢复运行时间计数器')
        }

        console.log('✅ KernelStore初始化完成')
      } catch (error) {
        console.error('❌ KernelStore初始化失败:', error)
      }
    }

    return {
      // 持久化数据
      version,
      newVersion,

      // 方法
      updateVersion,
      checkKernelVersion,
      startKernel,
      stopKernel,
      restartKernel,
      setupWebsocketConnection,
      initializeWebSocketConnectionsAsync,
      toggleIpVersion,
      initEventListeners,
      cleanupEventListeners,
      resetTemporaryData: () => {
        // 委托给运行时store处理
        const runtimeStore = useKernelRuntimeStore()
        runtimeStore.resetRuntimeData()
      },
      initializeStore,
    }
  },
  {
    // 现在只包含版本信息，可以安全持久化
    persist: true,
  },
)
