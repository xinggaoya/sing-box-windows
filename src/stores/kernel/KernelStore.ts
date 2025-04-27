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
            premium: true
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

        // 启动内核
        await tauriApi.kernel.startKernel()

        // 设置 WebSocket 连接 token
        const token = await tauriApi.proxy.getApiToken();
        const wsService = WebSocketService.getInstance();
        wsService.setToken(token);

        // 短暂延迟，等待内核和API服务启动
        await new Promise(resolve => setTimeout(resolve, 1000))
        
        // 尝试建立所有 WebSocket 连接
        const allConnected = await wsService.checkAllConnections();
        if (!allConnected) {
          console.warn('部分 WebSocket 连接失败，但仍会继续运行');
        }

        // 设置运行状态
        appStore.setRunningState(true)

        // 通知内核状态变更
        mitt.emit('kernel-started')

        return true
      } catch (error) {
        console.error('启动内核失败:', error)
        return false
      }
    }

    // 停止内核
    const stopKernel = async () => {
      try {
        // 清理计时器和事件监听器
        clearTimers()
        cleanupEventListeners()

        // 断开所有 WebSocket 连接
        const wsService = WebSocketService.getInstance();
        await wsService.disconnectAll();

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
        await new Promise(resolve => setTimeout(resolve, 500))

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

    return {
      version,
      newVersion,
      memory,
      uptime,
      startKernel,
      stopKernel,
      restartKernel,
      updateVersion,
      checkKernelVersion,
      toggleIpVersion,
      initEventListeners,
      cleanupEventListeners
    }
  },
  {
    persist: true,
  }
)
