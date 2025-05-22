import { defineStore } from 'pinia'
import { ref, onMounted, watch } from 'vue'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'
import mitt from '@/utils/mitt'
import { WebSocketService } from '@/services/websocket-service'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useMessage } from 'naive-ui'

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

    onMounted(async () => {
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
    })

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
    const startWebSocketCheck = () => {
      // 清除之前的超时处理
      if (connectionCheckTimeout) {
        clearTimeout(connectionCheckTimeout)
      }

      // 只在isRunning为true时才执行WebSocket检查
      if (!isRunning.value) return

      // 启动连接检查
      connectionCheckTimeout = window.setTimeout(async () => {
        try {
          // 首先检查内核是否真的在运行
          let kernelRunning = false
          try {
            // 这里使用导入的tauriApi而不是依赖注入，避免循环依赖问题
            const { invoke } = await import('@tauri-apps/api/core')
            kernelRunning = await invoke<boolean>('is_kernel_running').catch(() => false)
          } catch (e) {
            console.error('内核运行状态检查失败:', e)
          }

          // 如果内核没有运行，则设置状态为未运行并退出
          if (!kernelRunning) {
            console.log('检测到内核未运行，设置状态为stopped')
            isRunning.value = false
            wsConnected.value = false
            mitt.emit('process-status')
            return
          }

          // 获取WebSocket服务实例
          const wsService = WebSocketService.getInstance()
          // 检查WebSocket连接状态
          const trafficStore = useTrafficStore()
          const connectionStore = useConnectionStore()

          // 如果连接状态正常，则确认运行状态
          if (
            trafficStore.connectionState.connected ||
            connectionStore.connectionsState.connected
          ) {
            wsConnected.value = true
            return // 连接正常，不需要继续处理
          }

          // 最多尝试一次WebSocket重连
          console.log('内核运行但WebSocket未连接，尝试建立连接...')
          await Promise.allSettled([
            trafficStore.setupTrafficListener().catch(() => {}),
            connectionStore.setupConnectionsListener().catch(() => {}),
            connectionStore.setupMemoryListener().catch(() => {}),
          ])

          // 再次检查连接状态
          if (
            trafficStore.connectionState.connected ||
            connectionStore.connectionsState.connected
          ) {
            wsConnected.value = true
            console.log('WebSocket重连成功')
          } else {
            // 即使WebSocket连接失败，只要内核确认在运行，也保持运行状态
            console.warn('WebSocket连接失败，但内核仍在运行')
            // 不改变isRunning状态，保持内核运行状态
          }
        } catch (error) {
          console.error('WebSocket连接检查失败:', error)
          // 不自动设置内核状态，避免循环重试
        }
      }, 2000) // 给内核2秒时间启动
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
    }
  },
  {
    persist: true,
  },
)
