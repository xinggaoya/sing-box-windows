import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import mitt from '@/utils/mitt'

/**
 * WebSocket 连接状态接口
 */
export interface ConnectionState {
  connected: boolean
  connecting: boolean
  error: Error | null
}

/**
 * 简化的 WebSocket 服务类 - 单例模式
 * 负责与后端 WebSocket 中继服务的协调
 */
export class WebSocketService {
  private static instance: WebSocketService
  private isInitialized: boolean = false
  private isConnected: boolean = false

  private constructor() {
    this.initializeEventListeners()
  }

  public static getInstance(): WebSocketService {
    if (!WebSocketService.instance) {
      WebSocketService.instance = new WebSocketService()
    }
    return WebSocketService.instance
  }

  public static destroyInstance() {
    if (WebSocketService.instance) {
      WebSocketService.instance.destroy()
      WebSocketService.instance = null!
    }
  }

  private async initializeEventListeners() {
    if (this.isInitialized) return
    this.isInitialized = true

    try {
      // 监听内核就绪事件，自动启动 WebSocket 中继
      await listen('kernel-ready', async (event) => {
        console.log('🎉 收到内核就绪事件:', event.payload)
        await this.startWebSocketRelay()
      })

      // 监听各种数据事件并转发到内部事件系统
      await this.setupDataEventListeners()

      console.log('✅ WebSocket 服务事件监听器初始化完成')
    } catch (error) {
      console.error('❌ WebSocket 服务初始化失败:', error)
    }
  }

  private async setupDataEventListeners() {
    // 监听流量数据
    await listen('traffic-data', (event) => {
      mitt.emit('traffic-data', event.payload as Record<string, unknown>)
      // 收到数据说明连接正常，更新连接状态
      if (!this.isConnected) {
        console.log('📊 收到流量数据，更新连接状态为已连接')
        this.isConnected = true
        mitt.emit('ws-connected')
      }
    })

    // 监听内存数据
    await listen('memory-data', (event) => {
      mitt.emit('memory-data', event.payload as Record<string, unknown>)
      // 收到数据说明连接正常，更新连接状态
      if (!this.isConnected) {
        console.log('💾 收到内存数据，更新连接状态为已连接')
        this.isConnected = true
        mitt.emit('ws-connected')
      }
    })

    // 监听日志数据
    await listen('log-data', (event) => {
      mitt.emit('log-data', event.payload as { type: string; payload: string })
      // 收到数据说明连接正常，更新连接状态
      if (!this.isConnected) {
        console.log('📝 收到日志数据，更新连接状态为已连接')
        this.isConnected = true
        mitt.emit('ws-connected')
      }
    })

    // 监听连接数据
    await listen('connections-data', (event) => {
      mitt.emit('connections-data', event.payload as Record<string, unknown>)
      // 收到数据说明连接正常，更新连接状态
      if (!this.isConnected) {
        console.log('🔗 收到连接数据，更新连接状态为已连接')
        this.isConnected = true
        mitt.emit('ws-connected')
      }
    })

    // 监听WebSocket连接状态事件（如果后端发送的话）
    await listen('traffic-connection-state', (event) => {
      const state = event.payload as { connected?: boolean }
      console.log('📡 收到WebSocket连接状态事件:', state)
      if (state && state.connected) {
        this.isConnected = true
        mitt.emit('ws-connected')
      } else {
        this.isConnected = false
        mitt.emit('ws-disconnected')
      }
    })

    console.log('📡 数据事件监听器设置完成')
  }

  public async startWebSocketRelay(): Promise<boolean> {
    try {
      console.log('🔌 开始启动 WebSocket 中继服务...')

      // 动态获取AppStore中的API端口配置
      const { useAppStore } = await import('@/stores/app/AppStore')
      const appStore = useAppStore()

      // 等待数据恢复完成，确保端口配置正确
      await appStore.waitForDataRestore()

      await invoke('start_websocket_relay', {
        apiPort: appStore.apiPort,
      })

      console.log('✅ WebSocket 中继服务启动成功')
      this.isConnected = true
      mitt.emit('ws-connected')

      // 延迟检查数据流是否正常
      setTimeout(() => {
        if (this.isConnected) {
          console.log('🔍 WebSocket 中继启动 5 秒后，连接状态检查完成')
        }
      }, 5000)

      return true
    } catch (error) {
      console.error('❌ WebSocket 中继服务启动失败:', error)
      return false
    }
  }

  // 手动启动 WebSocket 中继（用于内核已运行但没有收到 kernel-ready 事件的情况）
  public async ensureWebSocketConnection(): Promise<boolean> {
    console.log('🔍 检查并确保 WebSocket 连接...')

    if (this.isConnected) {
      console.log('✅ WebSocket 连接状态正常')
      return true
    }

    console.log('⚠️ WebSocket 未连接，尝试手动启动中继服务...')
    return await this.startWebSocketRelay()
  }

  public async stopWebSocketRelay(): Promise<boolean> {
    try {
      this.isConnected = false
      mitt.emit('ws-disconnected')
      console.log('✅ WebSocket 连接已断开')
      return true
    } catch (error) {
      console.error('❌ WebSocket 断开失败:', error)
      return false
    }
  }

  public isWebSocketConnected(): boolean {
    return this.isConnected
  }

  public async manualReconnect(): Promise<boolean> {
    console.log('🔄 手动重新连接 WebSocket...')
    return await this.startWebSocketRelay()
  }

  public destroy() {
    this.isInitialized = false
    this.isConnected = false
    console.log('🧹 WebSocket 服务已销毁')
  }
}

export const webSocketService = WebSocketService.getInstance()
