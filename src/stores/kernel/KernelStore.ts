import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { eventService } from '@/services/event-service'
import { useAppStore } from '../app/AppStore'
import { useConnectionStore } from './ConnectionStore'
import { useTrafficStore } from './TrafficStore'
import { useLogStore } from './LogStore'
import { useKernelRuntimeStore } from './KernelRuntimeStore'

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

    // 事件监听器状态
    let eventListenersSetup = false

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

    // 启动内核（完整版本，包含状态检查）
    const startKernel = async () => {
      console.log('🚀 开始启动内核...')

      try {
        // 获取当前代理模式
        const proxyMode = appStore.proxyMode || 'manual'

        // 设置连接中状态
        appStore.setConnectingState(true)
        console.log('📡 正在启动内核进程...')

        // 确保数据Store已初始化，准备接收数据
        await ensureDataStoresInitialized()

        // 启动内核 - 传递API端口参数，后端会自动启动事件中继
        await tauriApi.kernel.startKernel(proxyMode, appStore.apiPort)
        console.log('✅ 内核进程启动成功，等待事件中继就绪...')

        // 等待并检查完整状态
        const isFullyReady = await pollKernelStatus(appStore.apiPort, 10)
        
        if (isFullyReady) {
          // 设置运行状态
          appStore.setRunningState(true)
          appStore.setConnectingState(false)

          console.log('🎉 内核启动完成 - 进程、API和WebSocket全部就绪')
          return true
        } else {
          throw new Error('内核启动超时，事件中继未能正常工作')
        }
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

        throw new Error(errorMessage)
      }
    }

    // 轮询检查内核完整状态
    const pollKernelStatus = async (apiPort: number, maxAttempts: number): Promise<boolean> => {
      console.log(`🔍 开始轮询检查内核状态，最大尝试次数: ${maxAttempts}`)
      
      for (let i = 0; i < maxAttempts; i++) {
        try {
          console.log(`📊 第 ${i + 1} 次状态检查...`)
          
          const status = await tauriApi.kernel.checkKernelStatus(apiPort)
          console.log(`📊 状态检查结果:`, status)
          
          const isFullyReady = status.process_running && 
                              status.api_ready && 
                              status.websocket_ready
          
          if (isFullyReady) {
            console.log('✅ 内核完全就绪！')
            return true
          }
          
          // 显示详细状态
          console.log(`⏳ 内核未完全就绪: 进程=${status.process_running}, API=${status.api_ready}, WebSocket=${status.websocket_ready}`)
          
        } catch (error) {
          console.warn(`⚠️ 第 ${i + 1} 次状态检查失败:`, error)
        }
        
        // 等待1秒再检查
        if (i < maxAttempts - 1) {
          await new Promise(resolve => setTimeout(resolve, 1000))
        }
      }
      
      console.error('❌ 内核状态轮询超时，未能完全就绪')
      return false
    }

    // 停止内核
    const stopKernel = async () => {
      try {
        // 清理计时器和事件监听器
        clearTimers()
        cleanupEventListeners()

        // 停止内核（后端会自动清理事件连接）
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
      if (eventListenersSetup) return

      try {
        // 监听内核就绪事件
        await eventService.onKernelReady(() => {
          console.log('🎉 收到内核就绪事件')
          appStore.setRunningState(true)
          appStore.setConnectingState(false)
        })

        // 更新版本信息
        await updateVersion()

        // 检查是否有新版本
        await checkKernelVersion()

        eventListenersSetup = true
        console.log('✅ KernelStore事件监听器初始化完成')
        return true
      } catch (error) {
        console.error('❌ 初始化事件监听器失败:', error)
        return false
      }
    }

    // 清理事件监听器
    const cleanupEventListeners = () => {
      if (!eventListenersSetup) return

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

      // 移除事件监听器
      eventService.removeEventListener('kernel-ready')

      eventListenersSetup = false
    }

    // 确保数据相关的Store已初始化
    const ensureDataStoresInitialized = async () => {
      try {
        // 动态导入StoreManager避免循环依赖
        const { storeManager } = await import('../StoreManager')

        // 预加载所有数据相关的Store
        await storeManager.preloadStores(['connection', 'traffic', 'log'])
        console.log('📦 数据Store预加载完成')

        // 立即手动初始化这些Store的事件监听器，确保在事件连接前就准备好
        try {
          const connectionStore = storeManager.getLoadedStore('connection')
          if (connectionStore) {
            // @ts-expect-error - Store类型推断问题，安全调用
            await connectionStore.initializeStore?.()
            console.log('📡 ConnectionStore事件监听器已初始化')
          }
        } catch (error) {
          console.warn('ConnectionStore初始化警告:', error)
        }

        try {
          const trafficStore = storeManager.getLoadedStore('traffic')
          if (trafficStore) {
            // @ts-expect-error - Store类型推断问题，安全调用
            await trafficStore.initializeStore?.()
            console.log('📊 TrafficStore事件监听器已初始化')
          }
        } catch (error) {
          console.warn('TrafficStore初始化警告:', error)
        }

        try {
          const logStore = storeManager.getLoadedStore('log')
          if (logStore) {
            // @ts-expect-error - Store类型推断问题，安全调用
            await logStore.initializeStore?.()
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

    // Store初始化方法
    const initializeStore = async () => {
      try {
        await initEventListeners()

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
      toggleIpVersion,
      initEventListeners,
      cleanupEventListeners,
      initializeStore,
    }
  },
  {
    // 现在只包含版本信息，可以安全持久化
    persist: true,
  },
)