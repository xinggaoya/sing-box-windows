import { defineStore } from 'pinia'
import { ref } from 'vue'
import { enable, disable } from '@tauri-apps/plugin-autostart'
import { useMessage } from 'naive-ui'
import { config } from '@/services/tauri-api'

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

      // WebSocket连接状态管理现在由后端直接处理，无需前端监听
      console.log('✅ AppStore初始化完成 - 使用Tauri事件系统')
    }

    // Store清理方法
    const cleanupStore = () => {
      if (connectionCheckTimeout) {
        clearTimeout(connectionCheckTimeout)
        connectionCheckTimeout = null
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

      // 代理模式变更事件现在通过Pinia响应式系统处理
      console.log('代理模式已切换到:', targetMode)
    }

    // 设置代理模式
    const setProxyMode = (mode: 'system' | 'tun' | 'manual') => {
      proxyMode.value = mode
    }

    // 更新端口配置
    const updatePorts = (newProxyPort: number, newApiPort: number) => {
      proxyPort.value = newProxyPort
      apiPort.value = newApiPort
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
      initializeStore,
      cleanupStore,
      markDataRestored,
      waitForDataRestore,
    }
  },
  {
    persist: true,
  },
)
