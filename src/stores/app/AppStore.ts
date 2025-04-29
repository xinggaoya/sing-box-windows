import { defineStore } from 'pinia'
import { ref, onMounted, watch } from 'vue'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'
import mitt from '@/utils/mitt'
import { WebSocketService } from '@/services/websocket-service'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'

// 代理模式类型
export type ProxyMode = 'system' | 'tun' | 'manual'

export const useAppStore = defineStore(
  'app',
  () => {
    // 应用运行状态
    const isRunning = ref(false)
    // WebSocket连接状态
    const wsConnected = ref(false)

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
    let connectionCheckTimeout: number | null = null;

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
        }

        // 发送进程状态变更事件
        mitt.emit('process-status')
      }
    }

    // 启动WebSocket连接检查
    const startWebSocketCheck = () => {
      // 清除之前的超时处理
      if (connectionCheckTimeout) {
        clearTimeout(connectionCheckTimeout)
      }

      // 启动连接检查
      connectionCheckTimeout = window.setTimeout(async () => {
        try {
          // 获取WebSocket服务实例
          const wsService = WebSocketService.getInstance()
          // 检查WebSocket连接状态
          const trafficStore = useTrafficStore()
          const connectionStore = useConnectionStore()

          // 如果连接状态正常，则确认运行状态
          if (trafficStore.connectionState.connected || connectionStore.connectionsState.connected) {
            wsConnected.value = true
          } else {
            // 如果连接状态异常，尝试重新建立连接
            console.log('内核启动但WebSocket未连接，尝试建立连接...')
            await Promise.allSettled([
              trafficStore.setupTrafficListener(),
              connectionStore.setupConnectionsListener(),
              connectionStore.setupMemoryListener()
            ])

            // 再次检查连接状态
            if (trafficStore.connectionState.connected || connectionStore.connectionsState.connected) {
              wsConnected.value = true
            } else {
              // 如果仍然无法连接，认为内核未正常启动
              console.error('无法建立WebSocket连接，内核可能未正常启动')
              isRunning.value = false
              wsConnected.value = false
              mitt.emit('process-status')
            }
          }
        } catch (error) {
          console.error('WebSocket连接检查失败:', error)
          isRunning.value = false
          wsConnected.value = false
          mitt.emit('process-status')
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

    return {
      isRunning,
      wsConnected,
      trayInstanceId,
      proxyMode,
      autoStartApp,
      autoStartKernel,
      preferIpv6,
      setRunningState,
      toggleAutoStart,
      switchProxyMode,
      startWebSocketCheck,
      setProxyMode
    }
  },
  {
    persist: true,
  }
)
