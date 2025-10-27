/**
 * 重构后的 KernelStore
 * 简化逻辑，专注于状态管理和用户交互
 */
import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { kernelService, type KernelStatus, type KernelConfig } from '@/services/kernel-service'
import { useAppStore } from '../app/AppStore'
import { useConnectionStore } from './ConnectionStore'
import { useTrafficStore } from './TrafficStore'
import { useLogStore } from './LogStore'
import { useKernelRuntimeStore } from './KernelRuntimeStore'

export const useKernelStore = defineStore(
  'kernel',
  () => {
    // 依赖的 stores
    const appStore = useAppStore()
    const connectionStore = useConnectionStore()
    const trafficStore = useTrafficStore()
    const logStore = useLogStore()
    const runtimeStore = useKernelRuntimeStore()

    // 响应式状态
    const status = ref<KernelStatus>({
      process_running: false,
      api_ready: false,
      websocket_ready: false,
      uptime_ms: 0,
      version: '',
      error: undefined,
    })

    const config = ref<KernelConfig>({
      proxy_mode: 'manual',
      api_port: 12081,
      proxy_port: 12080,
      prefer_ipv6: false,
      auto_start: false,
    })

    const isLoading = ref(false)
    const lastError = ref<string>('')

    // 计算属性
    const isRunning = computed(() => status.value.process_running)
    const isReady = computed(() => 
      status.value.process_running && 
      status.value.api_ready && 
      status.value.websocket_ready
    )
    const isStarting = computed(() => isLoading.value && !isRunning.value)
    const isStopping = computed(() => isLoading.value && isRunning.value)
    const uptime = computed(() => {
      const ms = status.value.uptime_ms || 0
      const seconds = Math.floor(ms / 1000)
      const minutes = Math.floor(seconds / 60)
      const hours = Math.floor(minutes / 60)
      
      if (hours > 0) {
        return `${hours}小时${minutes % 60}分钟`
      } else if (minutes > 0) {
        return `${minutes}分钟${seconds % 60}秒`
      } else {
        return `${seconds}秒`
      }
    })

    // 状态同步
    const syncStatus = async () => {
      try {
        status.value = await kernelService.getKernelStatus()
        
        // 同步到 appStore
        appStore.setRunningState(status.value.process_running)
        
        // 清除错误
        if (status.value.error) {
          lastError.value = status.value.error
        } else {
          lastError.value = ''
        }
      } catch (error) {
        console.error('同步内核状态失败:', error)
        lastError.value = error instanceof Error ? error.message : '状态同步失败'
      }
    }

    const syncConfig = async () => {
      try {
        // 等待AppStore数据恢复完成
        await appStore.waitForDataRestore(5000)
        
        // 从AppStore同步配置到KernelStore
        config.value = {
          proxy_mode: appStore.proxyMode as any,
          api_port: appStore.apiPort,
          proxy_port: appStore.proxyPort,
          prefer_ipv6: appStore.preferIpv6,
          auto_start: appStore.autoStartKernel,
        }
        
        console.log('📋 内核配置已同步:', {
          proxy_mode: config.value.proxy_mode,
          api_port: config.value.api_port,
          proxy_port: config.value.proxy_port,
          prefer_ipv6: config.value.prefer_ipv6,
          auto_start: config.value.auto_start,
        })
      } catch (error) {
        console.error('同步内核配置失败:', error)
      }
    }

    // 启动内核
    const startKernel = async (options?: { forceRestart?: boolean }) => {
      if (isLoading.value) {
        console.log('内核正在操作中，忽略启动请求')
        return false
      }

      isLoading.value = true
      lastError.value = ''

      try {
        console.log('🚀 开始启动内核...')
        
        // 准备启动选项
        const startOptions = {
          config: config.value,
          force_restart: options?.forceRestart || false,
          timeout_ms: 30000,
        }

        // 调用服务启动
        const result = await kernelService.startKernel(startOptions)
        
        if (result.success) {
          console.log('✅ 内核启动成功:', result.message)
          
          // 同步状态
          await syncStatus()
          
          // 启动数据收集
          await startDataCollection()
          
          return true
        } else {
          console.error('❌ 内核启动失败:', result.message)
          lastError.value = result.message
          return false
        }
      } catch (error) {
        console.error('❌ 内核启动异常:', error)
        lastError.value = error instanceof Error ? error.message : '启动异常'
        return false
      } finally {
        isLoading.value = false
      }
    }

    // 停止内核
    const stopKernel = async (options?: { force?: boolean }) => {
      if (isLoading.value) {
        console.log('内核正在操作中，忽略停止请求')
        return false
      }

      isLoading.value = true
      lastError.value = ''

      try {
        console.log('🛑 开始停止内核...')
        
        // 停止选项
        const stopOptions = {
          force: options?.force || false,
          timeout_ms: 10000,
        }

        // 调用服务停止
        const result = await kernelService.stopKernel(stopOptions)
        
        if (result.success) {
          console.log('✅ 内核停止成功:', result.message)
          
          // 同步状态
          await syncStatus()
          
          // 停止数据收集
          stopDataCollection()
          
          // 重置相关数据
          connectionStore.resetData()
          trafficStore.resetStats()
          runtimeStore.resetRuntimeData()
          
          return true
        } else {
          console.error('❌ 内核停止失败:', result.message)
          lastError.value = result.message
          return false
        }
      } catch (error) {
        console.error('❌ 内核停止异常:', error)
        lastError.value = error instanceof Error ? error.message : '停止异常'
        return false
      } finally {
        isLoading.value = false
      }
    }

    // 重启内核
    const restartKernel = async (options?: { force?: boolean }) => {
      console.log('🔄 开始重启内核...')
      
      const stopResult = await stopKernel({ force: options?.force })
      if (!stopResult) {
        return false
      }
      
      // 短暂等待
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      return startKernel({ forceRestart: options?.force })
    }

    // 切换代理模式
    const switchProxyMode = async (mode: 'system' | 'tun' | 'manual') => {
      if (isLoading.value) {
        console.log('内核正在操作中，忽略代理模式切换')
        return false
      }

      try {
        console.log('🔄 切换代理模式:', mode)
        
        const result = await kernelService.switchProxyMode(mode)
        
        if (result.success) {
          console.log('✅ 代理模式切换成功:', result.message)
          
          // 同步配置
          await syncConfig()
          
          // 如果内核正在运行，提示需要重启
          if (isRunning.value) {
            console.log('内核正在运行，需要重启以应用新的代理模式')
            // 可以自动重启或提示用户
            await restartKernel()
          }
          
          return true
        } else {
          console.error('❌ 代理模式切换失败:', result.message)
          lastError.value = result.message
          return false
        }
      } catch (error) {
        console.error('❌ 代理模式切换异常:', error)
        lastError.value = error instanceof Error ? error.message : '模式切换异常'
        return false
      }
    }

    // 切换IP版本偏好
    const toggleIpVersion = async (preferIpv6: boolean) => {
      if (isLoading.value) {
        console.log('内核正在操作中，忽略IP版本切换')
        return false
      }

      try {
        console.log('🔄 切换IP版本偏好:', preferIpv6)

        const result = await kernelService.toggleIpVersion(preferIpv6)

        if (result.success) {
          console.log('✅ IP版本切换成功:', result.message)

          // 同步配置
          await syncConfig()

          // 内核支持热修改配置，无需重启
          // 配置缓存已在 toggleIpVersion 中清除

          return true
        } else {
          console.error('❌ IP版本切换失败:', result.message)
          lastError.value = result.message
          return false
        }
      } catch (error) {
        console.error('❌ IP版本切换异常:', error)
        lastError.value = error instanceof Error ? error.message : 'IP版本切换异常'
        return false
      }
    }

    // 更新配置
    const updateConfig = async (updates: Partial<KernelConfig>) => {
      if (isLoading.value) {
        console.log('内核正在操作中，忽略配置更新')
        return false
      }

      try {
        console.log('🔧 更新内核配置:', updates)
        
        const newConfig = { ...config.value, ...updates }
        const result = await kernelService.updateKernelConfig(newConfig)
        
        if (result.success) {
          console.log('✅ 配置更新成功:', result.message)
          
          // 同步配置
          await syncConfig()
          
          // 如果关键配置改变且内核正在运行，需要重启
          const needRestart = updates.api_port || updates.proxy_port || updates.proxy_mode
          if (needRestart && isRunning.value) {
            await restartKernel()
          }
          
          return true
        } else {
          console.error('❌ 配置更新失败:', result.message)
          lastError.value = result.message
          return false
        }
      } catch (error) {
        console.error('❌ 配置更新异常:', error)
        lastError.value = error instanceof Error ? error.message : '配置更新异常'
        return false
      }
    }

    // 启动数据收集
    const startDataCollection = async () => {
      try {
        console.log('📊 启动数据收集...')
        
        // 初始化各个数据 store
        await connectionStore.initializeStore()
        await trafficStore.initializeStore()
        await logStore.initializeStore()
        
        // 启动运行时间计数
        runtimeStore.startUptimeCounter()
        
        console.log('✅ 数据收集启动完成')
      } catch (error) {
        console.error('❌ 数据收集启动失败:', error)
      }
    }

    // 停止数据收集
    const stopDataCollection = () => {
      try {
        console.log('📊 停止数据收集...')
        
        // 清理各个数据 store
        connectionStore.cleanupListeners()
        trafficStore.cleanupListeners()
        logStore.cleanupListeners()
        
        // 停止运行时间计数
        runtimeStore.stopUptimeCounter()
        
        console.log('✅ 数据收集停止完成')
      } catch (error) {
        console.error('❌ 数据收集停止失败:', error)
      }
    }

    // 健康检查
    const checkHealth = async () => {
      try {
        const result = await kernelService.checkKernelHealth()
        
        if (!result.healthy) {
          console.warn('⚠️ 内核健康检查发现问题:', result.issues)
          lastError.value = result.issues.join('; ')
        }
        
        return result
      } catch (error) {
        console.error('❌ 健康检查失败:', error)
        return { healthy: false, issues: ['健康检查失败'] }
      }
    }

    // 清除错误
    const clearError = () => {
      lastError.value = ''
    }

    // Store 初始化
    const initializeStore = async () => {
      try {
        console.log('🔧 初始化 KernelStore...')
        
        // 同步初始状态和配置
        await Promise.all([
          syncStatus(),
          syncConfig()
        ])
        
        // 设置事件监听
        setupEventListeners()
        
        // 如果内核正在运行，启动数据收集
        if (isRunning.value) {
          await startDataCollection()
        }
        
        console.log('✅ KernelStore 初始化完成')
      } catch (error) {
        console.error('❌ KernelStore 初始化失败:', error)
      }
    }

    // 设置事件监听
    const setupEventListeners = () => {
      // 监听状态变化
      kernelService.onKernelStatusChange((newStatus) => {
        status.value = newStatus
        appStore.setRunningState(newStatus.process_running)
      })

      // 监听内核就绪
      kernelService.onKernelReady(() => {
        console.log('🎉 收到内核就绪事件')
        appStore.setRunningState(true)
        startDataCollection()
      })

      // 监听内核错误
      kernelService.onKernelError((error) => {
        console.error('❌ 收到内核错误事件:', error)
        lastError.value = error
      })
    }

    // 自动状态同步
    let statusSyncInterval: NodeJS.Timeout | null = null

    const startStatusSync = () => {
      if (statusSyncInterval) {
        clearInterval(statusSyncInterval)
      }
      
      statusSyncInterval = setInterval(() => {
        if (isRunning.value) {
          syncStatus()
        }
      }, 5000) // 每5秒同步一次状态
    }

    const stopStatusSync = () => {
      if (statusSyncInterval) {
        clearInterval(statusSyncInterval)
        statusSyncInterval = null
      }
    }

    // 监听运行状态变化
    watch(isRunning, (running) => {
      if (running) {
        startStatusSync()
      } else {
        stopStatusSync()
      }
    })

    // 返回接口
    return {
      // 状态
      status,
      config,
      isLoading,
      lastError,
      
      // 计算属性
      isRunning,
      isReady,
      isStarting,
      isStopping,
      uptime,
      
      // 方法
      startKernel,
      stopKernel,
      restartKernel,
      switchProxyMode,
      toggleIpVersion,
      updateConfig,
      checkHealth,
      clearError,
      syncStatus,
      syncConfig,
      initializeStore,
      
      // 兼容旧接口
      hasVersionInfo: () => !!status.value.version,
      getVersionString: () => status.value.version || '',
      newVersion: ref(''),
      updateVersion: async () => {
        const version = await kernelService.getKernelVersion()
        status.value.version = version
        return true
      },
      checkKernelVersion: async () => {
        // 检查更新逻辑
        return true
      },
    }
  }
)