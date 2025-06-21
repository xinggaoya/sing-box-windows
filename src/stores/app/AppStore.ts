import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'
import mitt from '@/utils/mitt'
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

    // 托盘实例ID - 由TrayStore使用
    const trayInstanceId = ref<string | null>(null)

    // 代理模式
    const proxyMode = ref<ProxyMode>('system')

    // 自动启动设置
    const autoStartApp = ref(false)
    const autoStartKernel = ref(false)

    // IP版本设置
    const preferIpv6 = ref(false)

    // 连接检查超时处理
    let connectionCheckTimeout: number | null = null

    // 端口配置
    const proxyPort = ref(12080) // 代理端口
    const apiPort = ref(12081) // API端口

    // Store初始化方法
    const initializeStore = async () => {
      autoStartApp.value = await isEnabled()

      // 添加对WebSocket连接状态的监听
      mitt.on('ws-connected', () => {
        console.log('WebSocket连接成功事件接收到')
        wsConnected.value = true
        // 如果状态不一致，更新运行状态
        if (!isRunning.value) {
          isRunning.value = true
          mitt.emit('process-status')
        }
      })

      mitt.on('ws-disconnected', () => {
        console.log('WebSocket连接断开事件接收到')
        wsConnected.value = false
        // 如果连接断开且状态是运行中，需要更新状态
        if (isRunning.value) {
          // 延迟一点时间再判断，避免短暂断开后自动重连的情况
          if (connectionCheckTimeout) {
            clearTimeout(connectionCheckTimeout)
          }
          connectionCheckTimeout = window.setTimeout(() => {
            // 再次检查，如果还是断开状态，则认为内核已停止
            if (!wsConnected.value) {
              mitt.emit('process-status')
            }
          }, 5000) // 5秒后再检查
        }
      })
    }

    // Store清理方法
    const cleanupStore = () => {
      mitt.off('ws-connected')
      mitt.off('ws-disconnected')
      if (connectionCheckTimeout) {
        clearTimeout(connectionCheckTimeout)
        connectionCheckTimeout = null
      }
    }

    // 应用运行状态变更
    const setRunningState = (state: boolean) => {
      if (isRunning.value !== state) {
        isRunning.value = state

        // 如果设置为运行中，启动WebSocket连接检查
        if (state) {
          startWebSocketCheck()
        } else {
          // 如果设置为停止，清除WebSocket连接
          wsConnected.value = false
          // 同时确保连接中状态也被清除
          isConnecting.value = false
        }

        // 发送进程状态变更事件
        mitt.emit('process-status')
      }
    }

    // 设置连接中状态
    const setConnectingState = (state: boolean) => {
      isConnecting.value = state
      // 发送状态变更事件
      mitt.emit('connecting-status-changed', state)
    }

    // 启动WebSocket连接检查
    const startWebSocketCheck = async (): Promise<boolean> => {
      try {
        // 动态导入WebSocketService避免循环依赖
        const { WebSocketService } = await import('@/services/websocket-service')
        const wsService = WebSocketService.getInstance()

        // 设置API端口
        wsService.setApiPort(apiPort.value)

        // 通过事件总线检查所有连接，而不是直接调用store方法
        const success = await wsService.checkAllConnections()

        if (success) {
          wsConnected.value = true
          console.log('WebSocket连接检查成功')
        } else {
          wsConnected.value = false
          console.log('WebSocket连接检查失败')
        }

        return success
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
        autoStartApp.value = enabled
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

      // 发出代理模式变更事件，通知其他组件
      mitt.emit('proxy-mode-changed')
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
      trayInstanceId,
      proxyMode,
      autoStartApp,
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
    }
  },
  {
    persist: true,
  },
)
