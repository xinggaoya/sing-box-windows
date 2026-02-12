import { listen } from '@tauri-apps/api/event'
import { APP_EVENTS, type AppEventName, type AppEventPayloads } from '@/constants/events'

/**
 * 简化的事件服务类 - 替换原有的WebSocket管理
 * 直接使用Tauri事件系统，不再需要复杂的WebSocket管理
 */
export class EventService {
  private static instance: EventService
  private eventListeners: Map<string, Map<number, () => void>> = new Map()
  private listenerId = 1

  private constructor() {}

  public static getInstance(): EventService {
    if (!EventService.instance) {
      EventService.instance = new EventService()
    }
    return EventService.instance
  }

  public static destroyInstance() {
    if (EventService.instance) {
      EventService.instance.destroy()
      EventService.instance = null!
    }
  }

  /**
   * 注册监听器并返回监听器 ID
   */
  private registerListener(eventName: string, unlisten: () => void): number {
    const id = this.listenerId++
    const listeners = this.eventListeners.get(eventName) ?? new Map<number, () => void>()
    listeners.set(id, unlisten)
    this.eventListeners.set(eventName, listeners)
    return id
  }

  /**
   * 移除某个事件下的单个监听器
   */
  private removeEventListenerById(eventName: string, id: number) {
    const listeners = this.eventListeners.get(eventName)
    if (!listeners) return

    const unlisten = listeners.get(id)
    if (!unlisten) return

    try {
      unlisten()
    } catch (error) {
      console.error(`移除事件监听器失败 ${eventName}#${id}:`, error)
    } finally {
      listeners.delete(id)
      if (listeners.size === 0) {
        this.eventListeners.delete(eventName)
      }
    }
  }

  /**
   * 监听流量数据事件
   */
  public async onTrafficData(
    callback: (data: AppEventPayloads[typeof APP_EVENTS.trafficData]) => void
  ): Promise<() => void> {
    const unlisten = await listen(APP_EVENTS.trafficData, (event) => {
      callback(event.payload as AppEventPayloads[typeof APP_EVENTS.trafficData])
    })
    const id = this.registerListener(APP_EVENTS.trafficData, unlisten)
    return () => this.removeEventListener(APP_EVENTS.trafficData, id)
  }

  /**
   * 监听内存数据事件
   */
  public async onMemoryData(
    callback: (data: AppEventPayloads[typeof APP_EVENTS.memoryData]) => void
  ): Promise<() => void> {
    const unlisten = await listen(APP_EVENTS.memoryData, (event) => {
      callback(event.payload as AppEventPayloads[typeof APP_EVENTS.memoryData])
    })
    const id = this.registerListener(APP_EVENTS.memoryData, unlisten)
    return () => this.removeEventListener(APP_EVENTS.memoryData, id)
  }

  /**
   * 监听日志数据事件
   */
  public async onLogData(
    callback: (data: AppEventPayloads[typeof APP_EVENTS.logData]) => void
  ): Promise<() => void> {
    const unlisten = await listen(APP_EVENTS.logData, (event) => {
      callback(event.payload as AppEventPayloads[typeof APP_EVENTS.logData])
    })
    const id = this.registerListener(APP_EVENTS.logData, unlisten)
    return () => this.removeEventListener(APP_EVENTS.logData, id)
  }

  /**
   * 监听连接数据事件
   */
  public async onConnectionsData(
    callback: (data: AppEventPayloads[typeof APP_EVENTS.connectionsData]) => void
  ): Promise<() => void> {
    const unlisten = await listen(APP_EVENTS.connectionsData, (event) => {
      callback(event.payload as AppEventPayloads[typeof APP_EVENTS.connectionsData])
    })
    const id = this.registerListener(APP_EVENTS.connectionsData, unlisten)
    return () => this.removeEventListener(APP_EVENTS.connectionsData, id)
  }

  /**
   * 监听内核就绪事件
   */
  public async onKernelReady(callback: () => void): Promise<() => void> {
    const unlisten = await listen(APP_EVENTS.kernelReady, () => {
      callback()
    })
    const id = this.registerListener(APP_EVENTS.kernelReady, unlisten)
    return () => this.removeEventListener(APP_EVENTS.kernelReady, id)
  }

  /**
   * 通用事件监听方法
   */
  public async on<K extends AppEventName>(
    eventName: K,
    callback: (data: AppEventPayloads[K]) => void
  ): Promise<() => void>
  public async on(eventName: string, callback: (data: unknown) => void): Promise<() => void>
  public async on(eventName: string, callback: (data: unknown) => void): Promise<() => void> {
    const unlisten = await listen(eventName, (event) => {
      callback(event.payload as never)
    })
    const id = this.registerListener(eventName, unlisten)

    // 返回取消监听的函数
    return () => this.removeEventListener(eventName, id)
  }

  /**
   * 移除特定事件监听器
   */
  public removeEventListener(eventName: AppEventName | string, id?: number) {
    if (typeof id === 'number') {
      this.removeEventListenerById(eventName, id)
      return
    }

    const listeners = this.eventListeners.get(eventName)
    if (!listeners) return

    listeners.forEach((unlisten, listenerId) => {
      try {
        unlisten()
      } catch (error) {
        console.error(`移除事件监听器失败 ${eventName}#${listenerId}:`, error)
      }
    })
    this.eventListeners.delete(eventName)
  }

  public removeAllEventListeners() {
    const eventNames = Array.from(this.eventListeners.keys())
    eventNames.forEach((eventName) => {
      const listeners = this.eventListeners.get(eventName)
      if (!listeners) return
      listeners.forEach((unlisten, listenerId) => {
        try {
          unlisten()
        } catch (error) {
          console.error(`移除事件监听器失败 ${eventName}#${listenerId}:`, error)
        }
      })
      this.eventListeners.delete(eventName)
    })
  }

  /**
   * 销毁事件服务
   */
  public destroy() {
    this.removeAllEventListeners()
    console.log('🧹 事件服务已销毁')
  }
}

export const eventService = EventService.getInstance()
