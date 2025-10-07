import { defineStore } from 'pinia'
import { ref } from 'vue'
import { enable, disable } from '@tauri-apps/plugin-autostart'
import { useMessage } from 'naive-ui'
import { config, tauriApi } from '@/services/tauri-api'
import { storageService, type AppConfig } from '@/services/backend-storage-service'

// 代理模式类型
export type ProxyMode = 'system' | 'tun' | 'manual'

export const useAppStore = defineStore(
  'app',
  () => {
    // 消息服务实例
    let messageInstance: ReturnType<typeof useMessage> | null = null

    // 设置消息服务实例
    const setMessageInstance = (instance: ReturnType<typeof useMessage>) => {
      messageInstance = instance
    }

    // 显示成功消息
    const showSuccessMessage = (content: string) => {
      if (messageInstance) {
        messageInstance.success(content)
      }
    }

    // 显示错误消息
    const showErrorMessage = (content: string) => {
      if (messageInstance) {
        messageInstance.error(content)
      }
    }

    // 显示警告消息
    const showWarningMessage = (content: string) => {
      if (messageInstance) {
        messageInstance.warning(content)
      }
    }

    // 显示信息消息
    const showInfoMessage = (content: string) => {
      if (messageInstance) {
        messageInstance.info(content)
      }
    }

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

    // 数据恢复完成标志 - 解决启动竞态条件
    const isDataRestored = ref(false)
    // 数据恢复Promise，用于等待恢复完成
    let dataRestorePromise: Promise<void> | null = null
    let dataRestoreResolve: (() => void) | null = null

    // 托盘实例ID - 由TrayStore使用
    const trayInstanceId = ref<string | null>(null)

    // 代理模式
    const proxyMode = ref<ProxyMode>('system')

    const autoStartKernel = ref(false)

    // IP版本设置
    const preferIpv6 = ref(false)

    // 连接检查超时处理
    let connectionCheckTimeout: number | null = null

    // 端口配置
    const proxyPort = ref(12080) // 代理端口
    const apiPort = ref(12081) // API端口

    // 从后端加载数据
    const loadFromBackend = async () => {
      try {
        console.log('📋 从后端加载应用配置...')
        const appConfig = await storageService.getAppConfig()
        
        // 更新响应式状态
        proxyMode.value = appConfig.proxy_mode as ProxyMode
        autoStartKernel.value = appConfig.auto_start_kernel
        preferIpv6.value = appConfig.prefer_ipv6
        proxyPort.value = appConfig.proxy_port
        apiPort.value = appConfig.api_port
        trayInstanceId.value = appConfig.tray_instance_id
        
        console.log('📋 应用配置加载完成：', {
          proxyMode: proxyMode.value,
          autoStartKernel: autoStartKernel.value,
          preferIpv6: preferIpv6.value,
          proxyPort: proxyPort.value,
          apiPort: apiPort.value,
          trayInstanceId: trayInstanceId.value,
        })
        
        markDataRestored()
      } catch (error) {
        console.error('从后端加载应用配置失败:', error)
        // 加载失败时使用默认值
        markDataRestored()
      }
    }

    // 保存配置到后端
    const saveToBackend = async () => {
      try {
        const config: Partial<AppConfig> = {
          proxy_mode: proxyMode.value,
          auto_start_kernel: autoStartKernel.value,
          prefer_ipv6: preferIpv6.value,
          proxy_port: proxyPort.value,
          api_port: apiPort.value,
          tray_instance_id: trayInstanceId.value,
        }
        
        await storageService.updateAppConfig(config)
        console.log('✅ 应用配置已保存到后端')
      } catch (error) {
        console.error('保存应用配置到后端失败:', error)
      }
    }

    // 初始化数据恢复Promise
    const initializeDataRestore = () => {
      if (!dataRestorePromise) {
        dataRestorePromise = new Promise<void>((resolve) => {
          dataRestoreResolve = resolve
        })
      }
    }

    // 标记数据恢复完成
    const markDataRestored = () => {
      console.log('📋 AppStore 数据恢复完成，端口配置：', {
        proxyPort: proxyPort.value,
        apiPort: apiPort.value,
      })
      isDataRestored.value = true
      if (dataRestoreResolve) {
        dataRestoreResolve()
        dataRestoreResolve = null
      }
    }

    // 等待数据恢复完成
    const waitForDataRestore = async (timeout = 5000): Promise<boolean> => {
      if (isDataRestored.value) {
        return true
      }

      if (!dataRestorePromise) {
        console.warn('⚠️ 数据恢复Promise未初始化，可能存在时序问题')
        return false
      }

      try {
        await Promise.race([
          dataRestorePromise,
          new Promise((_, reject) => {
            setTimeout(() => reject(new Error('数据恢复超时')), timeout)
          }),
        ])
        return true
      } catch (error) {
        console.error('等待数据恢复失败:', error)
        // 即使超时也标记为已恢复，使用当前值
        markDataRestored()
        return false
      }
    }

    // Store初始化方法
    const initializeStore = async () => {
      // 初始化数据恢复Promise
      initializeDataRestore()

      // 从后端加载数据
      await loadFromBackend()

      // 检测是否是开机自启动场景
      await detectAutostartScenario()

      // WebSocket连接状态管理现在由后端直接处理，无需前端监听
      console.log('✅ AppStore初始化完成 - 使用后端存储')
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
      } catch (_error) {
        console.warn('无法获取系统运行时间，使用应用启动时间估算')
        // 如果无法获取系统时间，使用性能时间估算
        return Math.floor(performance.now() / 1000)
      }
    }

    // 延迟启动内核（用于开机自启动场景）
    const delayedKernelStart = async (delayMs: number = 10000): Promise<boolean> => {
      console.log(`⏰ 开机自启动场景，延迟${delayMs/1000}秒后启动内核...`)
      
      return new Promise((resolve) => {
        autostartDelayTimer = setTimeout(async () => {
          try {
            console.log('🚀 延迟时间到，开始启动内核...')
            // 动态导入避免循环依赖
            const { useKernelStore } = await import('../kernel/KernelStore')
            const kernelStore = useKernelStore()
            const result = await kernelStore.startKernel()
            resolve(result)
          } catch (error) {
            console.error('延迟启动内核失败:', error)
            resolve(false)
          } finally {
            autostartDelayTimer = null
          }
        }, delayMs)
      })
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

    // 切换自动启动
    const toggleAutoStart = async (enabled: boolean) => {
      try {
        if (enabled) {
          await enable()
        } else {
          await disable()
        }
        
        // 更新状态并保存到后端
        autoStartKernel.value = enabled
        await saveToBackend()
      } catch (error) {
        console.error('切换自动启动失败:', error)
        throw error
      }
    }

    // 代理模式切换
    const switchProxyMode = async (targetMode: ProxyMode) => {
      // 如果当前模式与目标模式相同，则不需要切换
      if (proxyMode.value === targetMode) return

      // 更新状态
      proxyMode.value = targetMode

      // 保存到后端
      await saveToBackend()

      // 代理模式变更事件现在通过Pinia响应式系统处理
      console.log('代理模式已切换到:', targetMode)
    }

    // 设置代理模式
    const setProxyMode = async (mode: 'system' | 'tun' | 'manual') => {
      proxyMode.value = mode
      await saveToBackend()
    }

    // 更新端口配置
    const updatePorts = async (newProxyPort: number, newApiPort: number) => {
      proxyPort.value = newProxyPort
      apiPort.value = newApiPort
      await saveToBackend()
    }

    // 同步端口配置到sing-box配置文件
    const syncPortsToSingbox = async () => {
      try {
        await config.updateSingboxPorts(proxyPort.value, apiPort.value)
        console.log('端口配置已同步到sing-box配置文件')
      } catch (error) {
        console.error('同步端口配置到sing-box失败:', error)
        throw error
      }
    }

    // 设置IPv6偏好
    const setPreferIpv6 = async (prefer: boolean) => {
      preferIpv6.value = prefer
      await saveToBackend()
    }

    // 设置托盘实例ID
    const setTrayInstanceId = async (instanceId: string | null) => {
      trayInstanceId.value = instanceId
      await saveToBackend()
    }

    return {
      isRunning,
      wsConnected,
      isConnecting,
      isDataRestored,
      trayInstanceId,
      proxyMode,
      autoStartKernel,
      preferIpv6,
      proxyPort,
      apiPort,
      isAutostartScenario,
      setRunningState,
      setConnectingState,
      toggleAutoStart,
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
      setTrayInstanceId,
      initializeStore,
      cleanupStore,
      markDataRestored,
      waitForDataRestore,
      detectAutostartScenario,
      delayedKernelStart,
      loadFromBackend,
      saveToBackend,
    }
  },
  // 移除 persist 配置，现在使用后端存储
)