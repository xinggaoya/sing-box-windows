import { listen } from '@tauri-apps/api/event'

/**
 * 简化的事件服务类 - 替换原有的WebSocket管理
 * 直接使用Tauri事件系统，不再需要复杂的WebSocket管理
 */
export class EventService {
  private static instance: EventService
  private eventListeners: Map<string, () => void> = new Map()

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
   * 监听流量数据事件
   */
  public async onTrafficData(callback: (data: unknown) => void) {
    const unlisten = await listen('traffic-data', (event) => {
      callback(event.payload)
    })
    this.eventListeners.set('traffic-data', unlisten)
  }

  /**
   * 监听内存数据事件
   */
  public async onMemoryData(callback: (data: unknown) => void) {
    const unlisten = await listen('memory-data', (event) => {
      callback(event.payload)
    })
    this.eventListeners.set('memory-data', unlisten)
  }

  /**
   * 监听日志数据事件
   */
  public async onLogData(callback: (data: unknown) => void) {
    const unlisten = await listen('log-data', (event) => {
      callback(event.payload)
    })
    this.eventListeners.set('log-data', unlisten)
  }

  /**
   * 监听连接数据事件
   */
  public async onConnectionsData(callback: (data: unknown) => void) {
    const unlisten = await listen('connections-data', (event) => {
      callback(event.payload)
    })
    this.eventListeners.set('connections-data', unlisten)
  }

  /**
   * 监听内核就绪事件
   */
  public async onKernelReady(callback: () => void) {
    const unlisten = await listen('kernel-ready', () => {
      callback()
    })
    this.eventListeners.set('kernel-ready', unlisten)
  }

  /**
   * 通用事件监听方法
   */
  public async on(eventName: string, callback: (data: unknown) => void): Promise<() => void> {
    const unlisten = await listen(eventName, (event) => {
      callback(event.payload)
    })
    this.eventListeners.set(eventName, unlisten)
    
    // 返回取消监听的函数
    return () => {
      this.removeEventListener(eventName)
    }
  }

  /**
   * 移除特定事件监听器
   */
  public removeEventListener(eventName: string) {
    const unlisten = this.eventListeners.get(eventName)
    if (unlisten) {
      try {
        unlisten()
        this.eventListeners.delete(eventName)
      } catch (error) {
        console.error(`移除事件监听器失败 ${eventName}:`, error)
        // 即使出错也要从Map中删除，避免重复尝试
        this.eventListeners.delete(eventName)
      }
    }
  }

  /**
   * 移除所有事件监听器
   */
  public removeAllEventListeners() {
    const eventNames = Array.from(this.eventListeners.keys())
    eventNames.forEach((eventName) => {
      const unlisten = this.eventListeners.get(eventName)
      if (unlisten) {
        try {
          unlisten()
        } catch (error) {
          console.error(`移除事件监听器失败 ${eventName}:`, error)
        }
      }
    })
    this.eventListeners.clear()
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