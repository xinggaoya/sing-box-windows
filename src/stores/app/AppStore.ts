import { defineStore } from 'pinia'
import { ref } from 'vue'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import type { MessageApi } from 'naive-ui/es/message'
import { config as configApi, tauriApi } from '@/services/tauri'
import { useAppMessaging } from './composables/messaging'
import { createAppPersistence } from './composables/persistence'

// 代理模式类型
export type ProxyMode = 'system' | 'tun' | 'manual'

const DEFAULT_SYSTEM_PROXY_BYPASS =
  'localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*'
const DEFAULT_TUN_IPV4 = '172.19.0.1/30'
const DEFAULT_TUN_IPV6 = 'fdfe:dcba:9876::1/126'
const DEFAULT_TUN_MTU = 1500

export const useAppStore = defineStore(
  'app',
  () => {
    const messaging = useAppMessaging()

    const setMessageInstance = (instance: MessageApi) => {
      messaging.setMessageInstance(instance)
    }

    const {
      showSuccessMessage,
      showErrorMessage,
      showWarningMessage,
      showInfoMessage,
    } = messaging

    // 应用运行状态
    const isRunning = ref(false)
    // WebSocket连接状态
    const wsConnected = ref(false)
    // 连接中状态（正在启动内核但尚未完成连接）
    const isConnecting = ref(false)

    // 开机自启动检测
    const isAutostartScenario = ref(false)
    // 自动启动延迟计时器
    let autostartDelayTimer: ReturnType<typeof setTimeout> | null = null

    // 托盘实例ID - 由TrayStore使用
    const trayInstanceId = ref<string | null>(null)

    // 代理模式
    const proxyMode = ref<ProxyMode>('system')

    const autoStartKernel = ref(false)

    // 系统开机自启动设置
    const autoStartApp = ref(false)

    // IP版本设置
    const preferIpv6 = ref(false)

    // 连接检查超时处理
    let connectionCheckTimeout: number | null = null

    // 端口配置
    const proxyPort = ref(12080) // 代理端口
    const apiPort = ref(12081) // API端口
    const systemProxyBypass = ref(DEFAULT_SYSTEM_PROXY_BYPASS)
    const tunIpv4 = ref(DEFAULT_TUN_IPV4)
    const tunIpv6 = ref(DEFAULT_TUN_IPV6)
    const tunMtu = ref(DEFAULT_TUN_MTU)
    const tunAutoRoute = ref(true)
    const tunStrictRoute = ref(true)
    const tunStack = ref<'system' | 'gvisor' | 'mixed'>('mixed')

    const {
      isDataRestored,
      startInitialization,
      finishInitialization,
      loadFromBackend,
      saveToBackend,
      waitForDataRestore,
      waitForSaveCompletion,
      markDataRestored,
      stopAutoSave,
    } = createAppPersistence({
      proxyMode,
      autoStartKernel,
      autoStartApp,
      preferIpv6,
      proxyPort,
      apiPort,
      trayInstanceId,
      systemProxyBypass,
      tunIpv4,
      tunIpv6,
      tunMtu,
      tunAutoRoute,
      tunStrictRoute,
      tunStack,
    })

    // 同步开机自启设置与系统状态
    const syncAutoStartWithSystem = async () => {
      try {
        // 检查系统实际的自启状态
        const systemEnabled = await isEnabled()

        console.log('🔍 系统自启状态检查:', {
          databaseSetting: autoStartApp.value,
          systemActual: systemEnabled,
        })

        // 如果数据库中设置为启用，但系统未注册，则重新注册
        if (autoStartApp.value && !systemEnabled) {
          console.log('⚠️ 检测到数据库自启设置为true但系统未注册，正在重新注册...')
          await enable()
          console.log('✅ 系统开机自启已重新注册')
        }
        // 如果数据库中设置为禁用，但系统已注册，则取消注册
        else if (!autoStartApp.value && systemEnabled) {
          console.log('⚠️ 检测到数据库自启设置为false但系统已注册，正在取消注册...')
          await disable()
          console.log('✅ 系统开机自启已取消注册')
        }
        // 两者一致，无需操作
        else {
          console.log('✅ 数据库设置与系统状态一致，无需同步')
        }
      } catch (error) {
        console.error('同步开机自启状态失败:', error)
        // 不抛出错误，避免影响应用正常启动
      }
    }

    // Store初始化方法
    const initializeStore = async () => {
      startInitialization()

      try {
        await loadFromBackend()
        console.log('📋 AppStore 数据恢复完成，配置：', {
          proxyPort: proxyPort.value,
          apiPort: apiPort.value,
          autoStartKernel: autoStartKernel.value,
          autoStartApp: autoStartApp.value,
        })

        await detectAutostartScenario()

        // 同步开机自启设置与系统状态（修复更新后设置丢失的问题）
        await syncAutoStartWithSystem()

        console.log('✅ AppStore初始化完成 - 使用数据库存储')

        // 注意：自动启动内核的逻辑现在由 App.vue 统一处理
        // 这里只加载数据，不执行启动逻辑，避免重复

        await new Promise(resolve => setTimeout(resolve, 100))
      } finally {
        finishInitialization()
      }
    }

    // 检测开机自启动场景
    const detectAutostartScenario = async () => {
      try {
        // 检查系统启动时间
        const systemUptime = await getSystemUptime()
        const isRecentStartup = systemUptime < 180 // 3分钟内认为是开机自启动
        
        // 检查应用启动时间
        const appStartTime = Date.now() - performance.now()
        const isRecentAppStart = (Date.now() - appStartTime) < 30000 // 30秒内启动的应用
        
        isAutostartScenario.value = isRecentStartup && isRecentAppStart
        
        if (isAutostartScenario.value) {
          console.log(`🕐 检测到开机自启动场景: 系统运行${systemUptime}秒, 应用启动${Math.round((Date.now() - appStartTime) / 1000)}秒前`)
        } else {
          console.log(`🖥️ 检测到正常启动场景: 系统运行${systemUptime}秒, 应用启动${Math.round((Date.now() - appStartTime) / 1000)}秒前`)
        }
      } catch (error) {
        console.warn('检测开机自启动场景失败:', error)
        isAutostartScenario.value = false
      }
    }

    // 获取系统运行时间（秒）
    const getSystemUptime = async (): Promise<number> => {
      try {
        // 使用Tauri命令获取系统启动时间
        const uptime = await tauriApi.system.getSystemUptime()
        return Math.floor(uptime / 1000) // 转换为秒
      } catch (error) {
        console.warn('无法获取系统运行时间，使用应用启动时间估算:', error)
        // 如果无法获取系统时间，使用性能时间估算
        return Math.floor(performance.now() / 1000)
      }
    }

    // 检查网络连接状态
    const checkNetworkReady = async (timeoutMs: number = 5000): Promise<boolean> => {
      try {
        console.log('🌐 检查网络连接状态...')
        // 尝试访问一个可靠的地址来检查网络
        const controller = new AbortController()
        const timeoutId = setTimeout(() => controller.abort(), timeoutMs)

        try {
          // 使用 fetch 检查网络连接，优先使用轻量级请求
          const response = await fetch('https://1.1.1.1', {
            method: 'HEAD',
            mode: 'no-cors',
            signal: controller.signal,
          })
          clearTimeout(timeoutId)
          console.log('✅ 网络连接正常')
          return true
        } catch (networkError) {
          clearTimeout(timeoutId)
          // 即使外部网络不可达，本地网络可能已就绪
          console.log('⚠️ 外部网络不可达，但可能本地网络已就绪')
          return true
        }
      } catch (error) {
        console.warn('网络检查失败:', error)
        // 默认返回 true，避免网络检查失败影响内核启动
        return true
      }
    }

    // 延迟启动内核（用于开机自启动场景，支持重试）
    const delayedKernelStart = async (
      delayMs: number = 20000,
      maxRetries: number = 3
    ): Promise<boolean> => {
      console.log(`⏰ 开机自启动场景，首次延迟${delayMs/1000}秒后启动内核（最多${maxRetries}次尝试）...`)

      // 首次延迟
      await new Promise(resolve => setTimeout(resolve, delayMs))

      // 检查网络连接
      const networkReady = await checkNetworkReady()
      if (!networkReady) {
        console.warn('⚠️ 网络未就绪，可能无法成功启动内核')
      }

      // 尝试启动内核（带重试机制）
      for (let attempt = 1; attempt <= maxRetries; attempt++) {
        try {
          console.log(`🚀 第 ${attempt}/${maxRetries} 次尝试启动内核...`)

          // 动态导入避免循环依赖
          const { useKernelStore } = await import('../kernel/KernelStore')
          const kernelStore = useKernelStore()

          // 等待应用Store数据完全恢复
          await waitForDataRestore(5000)

          const result = await kernelStore.startKernel()

          if (result) {
            console.log(`✅ 第 ${attempt} 次尝试成功启动内核！`)
            return true
          } else {
            throw new Error(kernelStore.lastError || '内核启动返回false')
          }
        } catch (error) {
          console.error(`❌ 第 ${attempt} 次尝试失败:`, error)

          // 如果不是最后一次尝试，等待后重试
          if (attempt < maxRetries) {
            const retryDelay = delayMs * attempt // 递增延迟：20s, 40s, 60s
            console.log(`⏳ ${retryDelay/1000} 秒后进行第 ${attempt + 1} 次尝试...`)
            await new Promise(resolve => setTimeout(resolve, retryDelay))
          }
        }
      }

      console.error(`❌ 经过 ${maxRetries} 次尝试后，内核启动仍然失败`)
      return false
    }

    // Store清理方法
    const cleanupStore = () => {
      if (connectionCheckTimeout) {
        clearTimeout(connectionCheckTimeout)
        connectionCheckTimeout = null
      }

      if (autostartDelayTimer) {
        clearTimeout(autostartDelayTimer)
        autostartDelayTimer = null
      }

      stopAutoSave()
    }

    // 应用运行状态变更
    const setRunningState = (state: boolean) => {
      if (isRunning.value !== state) {
        isRunning.value = state

        if (state) {
          // 现在使用Tauri事件系统，无需手动检查WebSocket连接
          console.log('内核运行状态已设置，事件系统会自动处理连接')

          // 移除WebSocket连接检查，因为Tauri事件系统会自动处理
          setTimeout(async () => {
            console.log('📡 Tauri事件系统已激活，等待后端推送数据')
          }, 2000)
        } else {
          // 如果设置为停止，清除连接状态
          wsConnected.value = false
          // 同时确保连接中状态也被清除
          isConnecting.value = false
        }

        // 进程状态变更现在通过Pinia响应式系统处理
        console.log('进程状态已变更:', state)
      }
    }

    // 设置连接中状态
    const setConnectingState = (state: boolean) => {
      isConnecting.value = state
      // 连接状态变更现在通过Pinia响应式系统处理
      console.log('连接状态已变更:', state)
    }

    // 启动WebSocket连接检查 - 简化版本，主要依赖事件系统
    const startWebSocketCheck = async (): Promise<boolean> => {
      try {
        // 新的WebSocket服务是事件驱动的，由后端自动管理
        // 这里只需要记录日志，实际连接状态通过事件更新
        console.log('🔌 WebSocket 连接检查 - 依赖后端自动管理')

        // 如果当前状态是运行中，假设WebSocket会自动连接
        if (isRunning.value) {
          console.log('内核运行中，WebSocket 应该会自动连接')
          return true
        }

        return false
      } catch (error) {
        console.error('WebSocket连接检查出错:', error)
        wsConnected.value = false
        return false
      }
    }

    // 切换系统开机自启
    const toggleAutoStart = async (enabled: boolean) => {
      try {
        if (enabled) {
          await enable()
        } else {
          await disable()
        }

        // 更新并持久化系统自启动状态
        autoStartApp.value = enabled
        await waitForSaveCompletion()

        // 注意：这里不应该改变 autoStartKernel，因为这是两个独立的设置
        // 系统开机自启 ≠ 启动内核
        // 只保存系统自启动状态，autoStartKernel 的值由用户单独控制
      } catch (error) {
        console.error('切换系统开机自启失败:', error)

        // 检测已知的无害错误，功能实际生效时仍然抛出错误以保持一致性
        const errorMessage = String(error)
        const isHarmlessError = errorMessage.includes('os error 2') ||
                               errorMessage.includes('system') ||
                               errorMessage.includes('No such file or directory')

        if (isHarmlessError) {
          console.log('Autostart 插件已知的无害错误，功能已生效:', error)
          // 仍然抛出错误，因为调用者需要知道操作完成了
          // 但在上层UI中已经被处理为不显示错误
        }

        throw error
      }
    }

    // 切换自动启动内核设置
    const toggleAutoStartKernel = async (enabled: boolean) => {
      try {
        // 只更新 autoStartKernel 设置
        autoStartKernel.value = enabled
        await waitForSaveCompletion()
        console.log(`自动启动内核设置已${enabled ? '启用' : '禁用'}`)
      } catch (error) {
        console.error('切换自动启动内核设置失败:', error)
        throw error
      }
    }

    // 代理模式切换
    const switchProxyMode = async (targetMode: ProxyMode) => {
      // 如果当前模式与目标模式相同，则不需要切换
      if (proxyMode.value === targetMode) return

      // 更新状态
      proxyMode.value = targetMode

      // 保存会在 watch 中自动处理
      console.log('代理模式已切换到:', targetMode)
    }

    // 设置代理模式
    const setProxyMode = async (mode: 'system' | 'tun' | 'manual') => {
      proxyMode.value = mode
      // 保存会在 watch 中自动处理
    }

    // 更新端口配置
    const updatePorts = async (newProxyPort: number, newApiPort: number) => {
      proxyPort.value = newProxyPort
      apiPort.value = newApiPort
      // 保存会在 watch 中自动处理
    }

    const updateProxyAdvancedSettings = async (settings: {
      systemProxyBypass?: string
      tunIpv4?: string
      tunIpv6?: string
      tunMtu?: number
      tunAutoRoute?: boolean
      tunStrictRoute?: boolean
      tunStack?: 'system' | 'gvisor' | 'mixed'
    }) => {
      if (typeof settings.systemProxyBypass === 'string') {
        systemProxyBypass.value = settings.systemProxyBypass
      }
      if (typeof settings.tunIpv4 === 'string') {
        tunIpv4.value = settings.tunIpv4
      }
      if (typeof settings.tunIpv6 === 'string') {
        tunIpv6.value = settings.tunIpv6
      }
      if (typeof settings.tunMtu === 'number') {
        tunMtu.value = settings.tunMtu
      }
      if (typeof settings.tunAutoRoute === 'boolean') {
        tunAutoRoute.value = settings.tunAutoRoute
      }
      if (typeof settings.tunStrictRoute === 'boolean') {
        tunStrictRoute.value = settings.tunStrictRoute
      }
      if (settings.tunStack && ['system', 'gvisor', 'mixed'].includes(settings.tunStack)) {
        tunStack.value = settings.tunStack
      }

      await waitForSaveCompletion()
    }

    // 同步端口配置到sing-box配置文件
    const syncPortsToSingbox = async () => {
      try {
        await configApi.updateSingboxPorts(proxyPort.value, apiPort.value)
        console.log('端口配置已同步到sing-box配置文件')
      } catch (error) {
        console.error('同步端口配置到sing-box失败:', error)
        throw error
      }
    }

    // 设置IPv6偏好
    const setPreferIpv6 = async (prefer: boolean) => {
      preferIpv6.value = prefer
      // 保存会在 watch 中自动处理
    }

    // 设置托盘实例ID
    const setTrayInstanceId = async (instanceId: string | null) => {
      trayInstanceId.value = instanceId
      // 保存会在 watch 中自动处理
    }

    return {
      isRunning,
      wsConnected,
      isConnecting,
      isDataRestored,
      trayInstanceId,
      proxyMode,
      autoStartKernel,
      autoStartApp,
      preferIpv6,
      proxyPort,
      apiPort,
      systemProxyBypass,
      tunIpv4,
      tunIpv6,
      tunMtu,
      tunAutoRoute,
      tunStrictRoute,
      tunStack,
      isAutostartScenario,
      setRunningState,
      setConnectingState,
      toggleAutoStart,
      toggleAutoStartKernel,
      switchProxyMode,
      startWebSocketCheck,
      setProxyMode,
      setMessageInstance,
      showSuccessMessage,
      showErrorMessage,
      showWarningMessage,
      showInfoMessage,
      updatePorts,
      syncPortsToSingbox,
      setPreferIpv6,
      updateProxyAdvancedSettings,
      setTrayInstanceId,
      initializeStore,
      cleanupStore,
      markDataRestored,
      waitForDataRestore,
      detectAutostartScenario,
      delayedKernelStart,
      syncAutoStartWithSystem,
      checkNetworkReady,
      loadFromBackend,
      saveToBackend,
    }
  },
  // 移除 persist 配置，现在使用后端存储
)
